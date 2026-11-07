//! Direct3D shader translation. DXIL is the canonical input; DXC supplies the
//! optional HLSL frontend. SPIR-V is an intermediate representation only.

use dxil_spirv::{Converter, ParsedBlob, ShaderStage, binding::*};
use serde::Serialize;
use spirv_cross2::{
    Compiler, Module, compile::msl, reflect::ExecutionModeArguments, spirv, targets,
};
use std::{
    collections::BTreeMap,
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};

pub type Result<T> = std::result::Result<T, String>;

/// Stages supported by this first discrete-resource translation contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Stage {
    Vertex,
    Pixel,
    Compute,
}

impl Stage {
    fn profile(self) -> &'static str {
        match self {
            Self::Vertex => "vs_6_0",
            Self::Pixel => "ps_6_0",
            Self::Compute => "cs_6_0",
        }
    }
    fn model(self) -> spirv::ExecutionModel {
        match self {
            Self::Vertex => spirv::ExecutionModel::Vertex,
            Self::Pixel => spirv::ExecutionModel::Fragment,
            Self::Compute => spirv::ExecutionModel::GLCompute,
        }
    }
}

/// D3D register namespaces stay distinct even when their register numbers match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum RegisterClass {
    Cbv,
    Srv,
    Uav,
    Sampler,
}

/// Each resource uses one of Metal's independent binding namespaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MetalNamespace {
    Buffer,
    Texture,
    Sampler,
}

/// Runtime binding information; original register spaces are never discarded.
#[derive(Clone, Debug, Serialize)]
pub struct ResourceBinding {
    pub class: RegisterClass,
    pub space: u32,
    pub register: u32,
    pub kind: String,
    pub alignment: u32,
    pub count: u32,
    pub namespace: MetalNamespace,
    pub metal_index: u32,
    pub spirv_set: u32,
    pub spirv_binding: u32,
}

/// Source and runtime metadata returned together so callers cannot guess slots.
#[derive(Debug, Serialize)]
pub struct Translation {
    pub msl: String,
    pub entry_point: String,
    pub stage: Stage,
    pub threads_per_group: Option<[u32; 3]>,
    pub resources: Vec<ResourceBinding>,
    pub vertex_inputs: Vec<VertexInput>,
    pub dxil_spirv_version: (u32, u32, u32),
    pub requires_ray_query: bool,
    pub fragment_output_components: Vec<(u32, u32)>,
}

/// Original D3D vertex semantics mapped to Metal attribute locations.
#[derive(Clone, Debug, Serialize)]
pub struct VertexInput {
    pub semantic: String,
    pub semantic_index: u32,
    pub location: u32,
    pub rows: u32,
}

/// Compile HLSL to a real DXIL container using an explicitly selected DXC.
/// Compiler errors and warnings fail; no SPIR-V shortcut or MSL fallback exists.
pub fn compile_hlsl(dxc: &Path, source: &Path, entry: &str, stage: Stage) -> Result<Vec<u8>> {
    compile_hlsl_with_options(dxc, source, entry, stage, &HlslOptions::default())
}

/// Explicit compilation choices. Stripping a root signature selects reflected
/// discrete resource bindings; callers must supply the corresponding runtime.
#[derive(Default)]
pub struct HlslOptions<'a> {
    pub shader_model_6_5: bool,
    pub strip_root_signature: bool,
    pub defines: &'a [(&'a str, &'a str)],
}

pub fn compile_hlsl_with_options(
    dxc: &Path,
    source: &Path,
    entry: &str,
    stage: Stage,
    options: &HlslOptions<'_>,
) -> Result<Vec<u8>> {
    validate_entry(entry)?;
    let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
    let output = temp.path().join("shader.dxil");
    let profile = if options.shader_model_6_5 {
        stage.profile().replace("6_0", "6_5")
    } else {
        stage.profile().into()
    };
    let mut command = Command::new(dxc);
    command
        .args(["-T", &profile, "-E", entry, "-Ges", "-WX", "-Fo"])
        .arg(&output);
    if options.strip_root_signature {
        command.arg("-Qstrip_rootsignature");
    }
    for (name, value) in options.defines {
        validate_entry(name)?;
        command.arg("-D").arg(format!("{name}={value}"));
    }
    let result = command
        .arg(source)
        .output()
        .map_err(|e| format!("DXC: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "DXC failed ({}): {}{}",
            result.status,
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let bytes = std::fs::read(output).map_err(|e| format!("DXC output: {e}"))?;
    validate_container(&bytes)?;
    Ok(bytes)
}

fn validate_entry(entry: &str) -> Result<()> {
    if entry.is_empty()
        || entry.len() > 256
        || !entry
            .bytes()
            .enumerate()
            .all(|(i, b)| b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit()))
    {
        return Err("entry point must be a nonempty HLSL identifier".into());
    }
    Ok(())
}

// Bound-check the DXBC envelope before passing it to the native parser. This
// milestone accepts DXIL containers only, not legacy SHDR/SHEX or raw LLVM IR.
fn validate_container(bytes: &[u8]) -> Result<()> {
    let invalid = || "invalid or truncated DXIL container".to_string();
    let word = |i: usize| -> Result<usize> {
        let b = bytes.get(i..i + 4).ok_or_else(invalid)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()) as usize)
    };
    if bytes.len() < 32
        || bytes.len() > 16 * 1024 * 1024
        || &bytes[..4] != b"DXBC"
        || word(20)? != 1
        || word(24)? != bytes.len()
    {
        return Err(invalid());
    }
    let count = word(28)?;
    let header_end = 32usize
        .checked_add(count.checked_mul(4).ok_or_else(invalid)?)
        .ok_or_else(invalid)?;
    if header_end > bytes.len() {
        return Err(invalid());
    }
    let mut ranges = Vec::new();
    let mut dxil = 0;
    for i in 0..count {
        let start = word(32 + i * 4)?;
        let size = word(start.checked_add(4).ok_or_else(invalid)?)?;
        let end = start
            .checked_add(8)
            .and_then(|v| v.checked_add(size))
            .ok_or_else(invalid)?;
        if start < header_end
            || start % 4 != 0
            || end > bytes.len()
            || ranges.iter().any(|&(a, b)| start < b && a < end)
        {
            return Err(invalid());
        }
        ranges.push((start, end));
        match &bytes[start..start + 4] {
            b"DXIL" => {
                dxil += 1;
                let base = start + 8;
                if size < 24
                    || &bytes[base + 8..base + 12] != b"DXIL"
                    || word(base + 4)?.checked_mul(4) != Some(size)
                {
                    return Err(invalid());
                }
                let offset = word(base + 16)?;
                let length = word(base + 20)?;
                let bitcode = (base + 8).checked_add(offset).ok_or_else(invalid)?;
                if offset < 16
                    || length < 4
                    || bitcode.checked_add(length).is_none_or(|v| v > end)
                    || bytes.get(bitcode..bitcode + 4) != Some(b"BC\xc0\xde")
                {
                    return Err(invalid());
                }
            }
            b"SHDR" | b"SHEX" => {
                return Err("legacy DXBC shader instructions are not supported".into());
            }
            b"RTS0" => return Err("embedded root signatures are not supported yet".into()),
            _ => {}
        }
    }
    if dxil != 1 {
        return Err("expected exactly one DXIL program".into());
    }
    Ok(())
}

#[derive(Default)]
struct Bindings {
    resources: BTreeMap<(RegisterClass, u32, u32), ResourceBinding>,
    error: Option<String>,
    vertex_inputs: BTreeMap<u32, VertexInput>,
}

impl Bindings {
    fn remap(
        &mut self,
        class: RegisterClass,
        b: &D3dBinding,
        counter: bool,
    ) -> Option<VulkanBinding> {
        let namespace = match b.kind {
            ResourceKind::RawBuffer
            | ResourceKind::StructuredBuffer
            | ResourceKind::ConstantBuffer
            | ResourceKind::RtAccelerationStructure => MetalNamespace::Buffer,
            ResourceKind::Sampler => MetalNamespace::Sampler,
            ResourceKind::Texture1D
            | ResourceKind::Texture2D
            | ResourceKind::Texture2DMs
            | ResourceKind::Texture3D
            | ResourceKind::TextureCube
            | ResourceKind::Texture1DArray
            | ResourceKind::Texture2DArray
            | ResourceKind::Texture2DMsArray
            | ResourceKind::TextureCubeArray => MetalNamespace::Texture,
            _ => {
                self.error = Some(format!("unsupported resource kind: {:?}", b.kind));
                return None;
            }
        };
        if b.range_size == 0
            || b.range_size > 128
            || (b.range_size != 1 && namespace != MetalNamespace::Texture)
            || counter
            || b.register_space > u32::MAX / 4
            || b.register_index.checked_add(b.range_size - 1).is_none()
        {
            self.error = Some(
                "unbounded/non-texture descriptor arrays, UAV counters or excessive register ranges are unsupported"
                    .into(),
            );
            return None;
        }
        let set = b.register_space * 4 + class as u32;
        for (&(other_class, space, register), other) in &self.resources {
            if other_class == class
                && space == b.register_space
                && register != b.register_index
                && (register as u64) < b.register_index as u64 + b.range_size as u64
                && (b.register_index as u64) < register as u64 + other.count as u64
            {
                self.error = Some("overlapping D3D descriptor ranges".into());
                return None;
            }
        }
        self.resources
            .entry((class, b.register_space, b.register_index))
            .or_insert(ResourceBinding {
                class,
                space: b.register_space,
                register: b.register_index,
                kind: format!("{:?}", b.kind),
                alignment: b.alignment,
                count: b.range_size,
                namespace,
                metal_index: 0,
                spirv_set: set,
                spirv_binding: b.register_index,
            });
        Some(VulkanBinding {
            set,
            binding: b.register_index,
            root_constant_index: 0,
            bindless: Bindless {
                heap_root_offset: 0,
                use_heap: false,
            },
            descriptor_type: match b.kind {
                ResourceKind::RawBuffer | ResourceKind::StructuredBuffer => {
                    VulkanDescriptorType::Ssbo
                }
                ResourceKind::ConstantBuffer => VulkanDescriptorType::Ubo,
                _ => VulkanDescriptorType::Identity,
            },
        })
    }
}

/// Translate a compiled DXIL container to MSL 2.3. Unsupported stages/resources
/// fail explicitly. This does not implement D3D12 root signatures or DXR dispatch.
pub fn translate_dxil(bytes: &[u8], entry: &str) -> Result<Translation> {
    translate_dxil_with_options(bytes, entry, &MslOptions::default())
}

/// Target attachment component counts, when the host uses a wider attachment
/// than the source shader's output (e.g. original RGB display into RGBA Metal).
#[derive(Default)]
pub struct MslOptions<'a> {
    pub fragment_output_components: &'a [(u32, u32)],
}

pub fn translate_dxil_with_options(
    bytes: &[u8],
    entry: &str,
    options: &MslOptions<'_>,
) -> Result<Translation> {
    validate_entry(entry)?;
    validate_container(bytes)?;
    let blob = ParsedBlob::parse(bytes).map_err(|e| format!("DXIL parse: {e}"))?;
    let stage = match blob
        .shader_stage_for_entry(entry)
        .map_err(|e| e.to_string())?
    {
        ShaderStage::Vertex => Stage::Vertex,
        ShaderStage::Pixel => Stage::Pixel,
        ShaderStage::Compute => Stage::Compute,
        s => {
            return Err(format!(
                "missing entry point or unsupported shader stage: {s:?}"
            ));
        }
    };
    let mut converter = Converter::new(&blob).map_err(|e| e.to_string())?;
    // Discrete buffers start at byte zero; 4-byte alignment is sufficient for
    // DXIL raw/structured loads. No Vulkan descriptor-offset emulation is used.
    converter
        .add_option(&dxil_spirv::options::ConverterOption::SsboAlignment { alignment: 4 })
        .map_err(|e| e.to_string())?;
    converter
        .set_entry_point(entry)
        .map_err(|e| e.to_string())?;
    let bindings = Arc::new(Mutex::new(Bindings::default()));
    let state = bindings.clone();
    converter.set_cbv_remapper(move |b| {
        state
            .lock()
            .unwrap()
            .remap(RegisterClass::Cbv, b, false)
            .map(CbvVulkanBinding::Uniform)
    });
    let state = bindings.clone();
    converter.set_srv_remapper(move |b| {
        state
            .lock()
            .unwrap()
            .remap(RegisterClass::Srv, b, false)
            .map(|v| SrvVulkanBinding {
                buffer_binding: v,
                offset_binding: v,
            })
    });
    let state = bindings.clone();
    converter.set_uav_remapper(move |b| {
        state
            .lock()
            .unwrap()
            .remap(RegisterClass::Uav, &b.binding, b.has_counter)
            .map(|v| UavVulkanBinding {
                buffer_binding: v,
                counter_binding: v,
                offset_binding: v,
            })
    });
    let state = bindings.clone();
    converter.set_sampler_remapper(move |b| {
        state
            .lock()
            .unwrap()
            .remap(RegisterClass::Sampler, b, false)
    });
    let state = bindings.clone();
    converter.set_vertex_input_remapper(move |v| {
        state.lock().unwrap().vertex_inputs.insert(
            v.start_row,
            VertexInput {
                semantic: v.semantic.clone(),
                semantic_index: v.semantic_index,
                location: v.start_row,
                rows: v.rows,
            },
        );
        Some(VulkanVertexInput {
            location: v.start_row,
        })
    });
    let result = converter.run();
    let mut state = bindings.lock().unwrap();
    if let Some(error) = state.error.take() {
        return Err(error);
    }
    result.map_err(|e| format!("DXIL to SPIR-V: {e}"))?;
    let words = converter.compiled_spirv().map_err(|e| e.to_string())?;
    let mut compiler =
        Compiler::<targets::Msl>::new(Module::from_words(&words)).map_err(|e| e.to_string())?;
    let mut locations = std::collections::BTreeSet::new();
    for &(location, components) in options.fragment_output_components {
        if stage != Stage::Pixel
            || location >= 8
            || !(1..=4).contains(&components)
            || !locations.insert(location)
        {
            return Err("invalid fragment output component contract".into());
        }
        compiler
            .set_fragment_output_components(location, components)
            .map_err(|e| e.to_string())?;
    }
    let requires_ray_query = compiler
        .declared_capabilities()
        .map_err(|e| e.to_string())?
        .contains(&spirv::Capability::RayQueryKHR);
    let entries = compiler
        .entry_points()
        .map_err(|e| e.to_string())?
        .collect::<Vec<_>>();
    if entries.len() != 1 || entries[0].execution_model != stage.model() {
        return Err("unexpected converted SPIR-V entry points".into());
    }
    let spirv_entry = entries[0].name.to_string();
    let workgroup = if stage == Stage::Compute {
        match compiler
            .execution_mode_arguments(spirv::ExecutionMode::LocalSize)
            .map_err(|e| e.to_string())?
        {
            Some(ExecutionModeArguments::LocalSize { x, y, z }) if x > 0 && y > 0 && z > 0 => {
                Some([x, y, z])
            }
            _ => return Err("compute shader has no fixed thread group size".into()),
        }
    } else {
        None
    };
    let mut next = [0u32; 3];
    for resource in state.resources.values_mut() {
        let ns = resource.namespace as usize;
        let limit = [24, 128, 16][ns];
        if resource.count > limit - next[ns] {
            return Err(format!("too many {:?} resources", resource.namespace));
        }
        resource.metal_index = next[ns];
        next[ns] += resource.count;
        compiler
            .add_resource_binding(
                stage.model(),
                msl::ResourceBinding::from_qualified(resource.spirv_set, resource.spirv_binding),
                &msl::BindTarget {
                    buffer: resource.metal_index,
                    texture: resource.metal_index,
                    sampler: resource.metal_index,
                    count: std::num::NonZeroU32::new(resource.count),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    let fragment_output_components = options.fragment_output_components.to_vec();
    let pad_fragment_outputs = !fragment_output_components.is_empty();
    let mut options = msl::CompilerOptions::default();
    options.pad_fragment_output_components = pad_fragment_outputs;
    options.version = msl::MslVersion::new(2, 3, 0);
    let artifact = compiler
        .compile(&options)
        .map_err(|e| format!("SPIR-V to MSL: {e}"))?;
    let requirements = artifact.buffer_requirements();
    if requirements.needs_swizzle_buffer
        || requirements.needs_buffer_size_buffer
        || requirements.needs_output_buffer
        || requirements.needs_patch_output_buffer
        || requirements.needs_input_threadgroup_buffer
    {
        return Err("unsupported auxiliary Metal buffer requirement".into());
    }
    let entry_point = artifact
        .cleansed_entry_point_name(spirv_entry.as_str(), stage.model())
        .map_err(|e| e.to_string())?
        .ok_or("MSL entry point missing")?
        .to_string();
    Ok(Translation {
        msl: artifact.to_string(),
        entry_point,
        stage,
        threads_per_group: workgroup,
        resources: state.resources.values().cloned().collect(),
        vertex_inputs: state.vertex_inputs.values().cloned().collect(),
        dxil_spirv_version: dxil_spirv::version(),
        requires_ray_query,
        fragment_output_components,
    })
}
