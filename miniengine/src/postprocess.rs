//! Unmodified MiniEngine tone/display shaders compiled by the open shader chain.
use metal::*;

struct Shader {
    source: String,
    entry: String,
    resources: Vec<serde_json::Value>,
}
impl Shader {
    fn load(text: &str, stage: &str) -> Result<Self, String> {
        let data: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if data["stage"] != stage || data["requires_ray_query"] != false {
            return Err("wrong postprocess shader stage/capability".into());
        }
        if stage == "Compute" && data["threads_per_group"] != serde_json::json!([8, 8, 1]) {
            return Err("wrong tone-map thread group".into());
        }
        let resources = data["resources"]
            .as_array()
            .ok_or("missing postprocess bindings")?
            .clone();
        Ok(Self {
            source: data["msl"]
                .as_str()
                .ok_or("missing postprocess MSL")?
                .into(),
            entry: data["entry_point"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("missing postprocess entry")?
                .into(),
            resources,
        })
    }
    fn slot(&self, class: &str, register: u32, namespace: &str) -> Result<u64, String> {
        let found: Vec<_> = self
            .resources
            .iter()
            .filter(|r| {
                r["class"] == class
                    && r["register"] == register
                    && r["space"] == 0
                    && r["namespace"] == namespace
            })
            .collect();
        if found.len() != 1 {
            return Err("missing/duplicate postprocess binding".into());
        }
        found[0]["metal_index"]
            .as_u64()
            .filter(|&v| {
                v < match namespace {
                    "Buffer" => 24,
                    "Texture" => 128,
                    _ => 16,
                }
            })
            .ok_or_else(|| "invalid postprocess Metal slot".into())
    }
    fn function(&self, device: &DeviceRef) -> Result<Function, String> {
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        device
            .new_library_with_source(&self.source, &options)?
            .get_function(&self.entry, None)
    }
}
pub(crate) struct Postprocess {
    pub(crate) input: Texture,
    luma: Texture,
    bloom: Texture,
    exposure: Buffer,
    sampler: SamplerState,
    tone: ComputePipelineState,
    tone_slots: [u64; 6],
    display: RenderPipelineState,
    copy: RenderPipelineState,
    upsample: RenderPipelineState,
    diagnostic_present: RenderPipelineState,
    diagnostic_slots: [u64; 2],
    display_slot: u64,
    copy_slot: u64,
    upsample_slots: [u64; 2],
}
fn image(device: &DeviceRef, width: u32, height: u32, format: MTLPixelFormat) -> Texture {
    let d = TextureDescriptor::new();
    d.set_texture_type(MTLTextureType::D2);
    d.set_width(width as u64);
    d.set_height(height as u64);
    d.set_pixel_format(format);
    d.set_storage_mode(MTLStorageMode::Private);
    d.set_usage(MTLTextureUsage::ShaderRead | MTLTextureUsage::ShaderWrite);
    device.new_texture(&d)
}
fn render_pipeline(
    device: &DeviceRef,
    vertex: &FunctionRef,
    pixel: &FunctionRef,
    format: MTLPixelFormat,
    rgb_only: bool,
) -> Result<RenderPipelineState, String> {
    let d = RenderPipelineDescriptor::new();
    d.set_vertex_function(Some(vertex));
    d.set_fragment_function(Some(pixel));
    let a = d.color_attachments().object_at(0).unwrap();
    a.set_pixel_format(format);
    // Original float3 shaders target RGB formats. Preserve an explicit opaque
    // alpha when using a four-channel Metal render target for capture/windowing.
    if rgb_only {
        a.set_write_mask(
            MTLColorWriteMask::Red | MTLColorWriteMask::Green | MTLColorWriteMask::Blue,
        );
    }
    device.new_render_pipeline_state(&d)
}
impl Postprocess {
    pub(crate) fn new(device: &DeviceRef, width: u32, height: u32) -> Result<Self, String> {
        let tone = Shader::load(
            include_str!("../../shader/samples/miniengine/tone.generated.json"),
            "Compute",
        )?;
        let display = Shader::load(
            include_str!("../../shader/samples/miniengine/display.generated.json"),
            "Pixel",
        )?;
        let screen = Shader::load(
            include_str!("../../shader/samples/miniengine/screen.generated.json"),
            "Vertex",
        )?;
        let copy = Shader::load(
            include_str!("../../shader/samples/miniengine/copy.generated.json"),
            "Pixel",
        )?;
        let upsample = Shader::load(
            include_str!("../../shader/samples/miniengine/upsample.generated.json"),
            "Pixel",
        )?;
        let diagnostic = Shader::load(
            include_str!("../../shader/samples/miniengine/diagnostic_present.generated.json"),
            "Pixel",
        )?;
        if tone.resources.len() != 6
            || display.resources.len() != 1
            || !screen.resources.is_empty()
            || copy.resources.len() != 1
            || upsample.resources.len() != 2
        {
            return Err("unexpected postprocess resource contract".into());
        }
        let tone_slots = [
            tone.slot("Cbv", 0, "Buffer")?,
            tone.slot("Srv", 0, "Buffer")?,
            tone.slot("Srv", 1, "Texture")?,
            tone.slot("Uav", 0, "Texture")?,
            tone.slot("Uav", 1, "Texture")?,
            tone.slot("Sampler", 0, "Sampler")?,
        ];
        let vertex = screen.function(device)?;
        let display_function = display.function(device)?;
        let copy_function = copy.function(device)?;
        let upsample_function = upsample.function(device)?;
        let diagnostic_function = diagnostic.function(device)?;
        let tone_function = tone.function(device)?;
        let desc = SamplerDescriptor::new();
        desc.set_min_filter(MTLSamplerMinMagFilter::Linear);
        desc.set_mag_filter(MTLSamplerMinMagFilter::Linear);
        desc.set_address_mode_s(MTLSamplerAddressMode::ClampToEdge);
        desc.set_address_mode_t(MTLSamplerAddressMode::ClampToEdge);
        desc.set_address_mode_r(MTLSamplerAddressMode::ClampToEdge);
        desc.set_mip_filter(MTLSamplerMipFilter::Linear);
        let bd = TextureDescriptor::new();
        bd.set_texture_type(MTLTextureType::D2);
        bd.set_width(1);
        bd.set_height(1);
        bd.set_pixel_format(MTLPixelFormat::RGBA32Float);
        bd.set_storage_mode(MTLStorageMode::Shared);
        bd.set_usage(MTLTextureUsage::ShaderRead);
        let bloom = device.new_texture(&bd);
        bloom.replace_region(
            MTLRegion::new_2d(0, 0, 1, 1),
            0,
            [0f32; 4].as_ptr().cast(),
            16,
        );
        let exposure = device.new_buffer_with_data(
            [2f32].as_ptr().cast(),
            4,
            MTLResourceOptions::StorageModeShared,
        );
        Ok(Self {
            input: image(device, width, height, MTLPixelFormat::RGBA32Float),
            luma: image(device, width, height, MTLPixelFormat::R32Float),
            bloom,
            exposure,
            sampler: device.new_sampler(&desc),
            tone: device.new_compute_pipeline_state_with_function(&tone_function)?,
            tone_slots,
            display: render_pipeline(
                device,
                &vertex,
                &display_function,
                MTLPixelFormat::RGBA8Unorm,
                true,
            )?,
            copy: render_pipeline(
                device,
                &vertex,
                &copy_function,
                MTLPixelFormat::RGBA8Unorm,
                false,
            )?,
            upsample: render_pipeline(
                device,
                &vertex,
                &upsample_function,
                MTLPixelFormat::BGRA8Unorm,
                true,
            )?,
            diagnostic_present: render_pipeline(
                device,
                &vertex,
                &diagnostic_function,
                MTLPixelFormat::BGRA8Unorm,
                false,
            )?,
            diagnostic_slots: [
                diagnostic.slot("Srv", 0, "Texture")?,
                diagnostic.slot("Sampler", 0, "Sampler")?,
            ],
            display_slot: display.slot("Srv", 0, "Texture")?,
            copy_slot: copy.slot("Srv", 0, "Texture")?,
            upsample_slots: [
                upsample.slot("Srv", 0, "Texture")?,
                upsample.slot("Sampler", 0, "Sampler")?,
            ],
        })
    }
    pub(crate) fn resize(&mut self, device: &DeviceRef, width: u32, height: u32) {
        self.input = image(device, width, height, MTLPixelFormat::RGBA32Float);
        self.luma = image(device, width, height, MTLPixelFormat::R32Float);
    }
    pub(crate) fn encode(&self, command: &CommandBufferRef, output: &TextureRef, shaded: bool) {
        if shaded {
            let constants = [
                1. / self.input.width() as f32,
                1. / self.input.height() as f32,
                0.,
                0.,
                0.,
                0.,
                0.,
                0.,
            ]; // Bloom remains unimplemented: strength=0, black texture.
            let encoder = command.new_compute_command_encoder();
            encoder.set_compute_pipeline_state(&self.tone);
            encoder.set_bytes(self.tone_slots[0], 32, constants.as_ptr().cast());
            encoder.set_buffer(self.tone_slots[1], Some(&self.exposure), 0);
            encoder.set_texture(self.tone_slots[2], Some(&self.bloom));
            encoder.set_texture(self.tone_slots[3], Some(&self.input));
            encoder.set_texture(self.tone_slots[4], Some(&self.luma));
            encoder.set_sampler_state(self.tone_slots[5], Some(&self.sampler));
            // No barriers in the original shader; Metal nonuniform dispatch
            // runs exactly the logical extent, including odd-sized captures.
            encoder.dispatch_threads(
                MTLSize::new(output.width(), output.height(), 1),
                MTLSize::new(8, 8, 1),
            );
            encoder.end_encoding();
        }
        let pass = RenderPassDescriptor::new();
        let a = pass.color_attachments().object_at(0).unwrap();
        a.set_texture(Some(output));
        a.set_load_action(MTLLoadAction::Clear);
        a.set_clear_color(MTLClearColor::new(0., 0., 0., 1.));
        a.set_store_action(MTLStoreAction::Store);
        let encoder = command.new_render_command_encoder(pass);
        encoder.set_render_pipeline_state(if shaded { &self.display } else { &self.copy });
        encoder.set_fragment_texture(
            if shaded {
                self.display_slot
            } else {
                self.copy_slot
            },
            Some(&self.input),
        );
        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 3);
        encoder.end_encoding();
    }
    pub(crate) fn present_shaded(&self, command: &CommandBufferRef, output: &TextureRef) {
        let pass = RenderPassDescriptor::new();
        let a = pass.color_attachments().object_at(0).unwrap();
        a.set_texture(Some(output));
        a.set_load_action(MTLLoadAction::Clear);
        a.set_clear_color(MTLClearColor::new(0., 0., 0., 1.));
        a.set_store_action(MTLStoreAction::Store);
        let encoder = command.new_render_command_encoder(pass);
        encoder.set_render_pipeline_state(&self.upsample);
        encoder.set_fragment_texture(self.upsample_slots[0], Some(&self.input));
        encoder.set_fragment_sampler_state(self.upsample_slots[1], Some(&self.sampler));
        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 3);
        encoder.end_encoding();
    }
    pub(crate) fn present_diagnostic(
        &self,
        command: &CommandBufferRef,
        output: &TextureRef,
        source: &TextureRef,
    ) {
        let pass = RenderPassDescriptor::new();
        let a = pass.color_attachments().object_at(0).unwrap();
        a.set_texture(Some(output));
        a.set_load_action(MTLLoadAction::DontCare);
        a.set_store_action(MTLStoreAction::Store);
        let encoder = command.new_render_command_encoder(pass);
        encoder.set_render_pipeline_state(&self.diagnostic_present);
        encoder.set_fragment_texture(self.diagnostic_slots[0], Some(source));
        encoder.set_fragment_sampler_state(self.diagnostic_slots[1], Some(&self.sampler));
        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 3);
        encoder.end_encoding();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_shader_metadata_rejects_wrong_stages_and_missing_resources() {
        let source = include_str!("../../shader/samples/miniengine/tone.generated.json");
        let shader = Shader::load(source, "Compute").unwrap();
        assert!(shader.slot("Srv", 0, "Buffer").is_ok());
        assert!(Shader::load(source, "Pixel").is_err());
        assert!(Shader::load("{}", "Compute").is_err());
        assert!(shader.slot("Srv", 900, "Buffer").is_err());
        assert!(shader.slot("Srv", 0, "Texture").is_err());
    }
    #[test]
    fn original_tone_display_and_luminance_execute_at_an_odd_extent() {
        objc::rc::autoreleasepool(|| {
            let host = pollster::block_on(alkyhol_d3d12::D3D12CreateDevice(false)).unwrap();
            eprintln!(
                "original tone/display compute+raster adapter: {:?}",
                host.adapter_info()
            );
            let device = host.metal_device();
            let (width, height) = (13u32, 7u32);
            let post = Postprocess::new(device, width, height).unwrap();
            let row = 256u64;
            let size = row * height as u64;
            let mut input = vec![0f32; (size / 4) as usize];
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let a = (x + y * width as usize) as f32 / 16.;
                    let i = y * 64 + x * 4;
                    input[i..i + 4].copy_from_slice(&[a, a * 0.5, a * 2., 1.]);
                }
            }
            let upload = device.new_buffer_with_data(
                input.as_ptr().cast(),
                size,
                MTLResourceOptions::StorageModeShared,
            );
            let descriptor = TextureDescriptor::new();
            descriptor.set_texture_type(MTLTextureType::D2);
            descriptor.set_width(width as u64);
            descriptor.set_height(height as u64);
            descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
            descriptor.set_storage_mode(MTLStorageMode::Private);
            descriptor.set_usage(MTLTextureUsage::RenderTarget);
            let output = device.new_texture(&descriptor);
            let pixels = device.new_buffer(size, MTLResourceOptions::StorageModeShared);
            let luma = device.new_buffer(size, MTLResourceOptions::StorageModeShared);
            let command = host.metal_queue().new_command_buffer();
            let encoder = command.new_blit_command_encoder();
            encoder.copy_from_buffer_to_texture(
                &upload,
                0,
                row,
                size,
                MTLSize::new(width as u64, height as u64, 1),
                &post.input,
                0,
                0,
                MTLOrigin::default(),
                MTLBlitOption::None,
            );
            encoder.end_encoding();
            post.encode(command, &output, true);
            let encoder = command.new_blit_command_encoder();
            for (texture, buffer) in [(&output, &pixels), (&post.luma, &luma)] {
                encoder.copy_from_texture_to_buffer(
                    texture,
                    0,
                    0,
                    MTLOrigin::default(),
                    MTLSize::new(width as u64, height as u64, 1),
                    buffer,
                    0,
                    row,
                    size,
                    MTLBlitOption::None,
                );
            }
            encoder.end_encoding();
            alkyhol_d3d12::complete(command).unwrap();
            let got = unsafe {
                std::slice::from_raw_parts(pixels.contents().cast::<u8>(), size as usize)
            };
            let luminance = unsafe {
                std::slice::from_raw_parts(luma.contents().cast::<f32>(), (size / 4) as usize)
            };
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let i = y * 64 + x * 4;
                    let pixel = y * row as usize + x * 4;
                    let mut sdr = [0f64; 3];
                    for c in 0..3 {
                        let hdr = input[i + c] as f64 * 2.;
                        let shoulder = hdr * hdr.sqrt();
                        sdr[c] = shoulder / (shoulder + (4f64 / 27.).sqrt());
                        let encoded = if sdr[c] < 0.0031308 {
                            sdr[c] * 12.92
                        } else {
                            1.055 * sdr[c].powf(1. / 2.4) - 0.055
                        };
                        assert!(
                            (got[pixel + c] as f64 - encoded * 255.).abs() <= 1.,
                            "tone/display pixel {x},{y} channel {c}"
                        );
                    }
                    assert_eq!(got[pixel + 3], 255, "opaque capture alpha");
                    let expected =
                        ((sdr[0] * 0.212671 + sdr[1] * 0.715160 + sdr[2] * 0.072169) * 15. + 1.)
                            .log2()
                            * 0.25;
                    assert!(
                        (luminance[y * 64 + x] as f64 - expected).abs() < 2e-6,
                        "luma {x},{y}"
                    );
                }
            }
        });
    }
}
