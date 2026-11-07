//! Native Metal triangle BLAS/TLAS. Geometry is non-indexed float3 in original primitive order.
use crate::{Device, backend, complete};
use alkyhol::graphics::{GraphicsError, Result};
use metal::*;

#[derive(Clone, Copy, Debug)]
pub struct TriangleInstance {
    pub mesh: u32,
    /// Column-major affine packed float4x3, matching Metal's native descriptor.
    pub transform: [[f32; 3]; 4],
    pub mask: u32,
    /// Disable triangle culling regardless of ray flags (Sakura uses double-sided rays).
    pub disable_culling: bool,
}
pub struct TriangleScene {
    pub acceleration: AccelerationStructure,
    bottom_levels: Vec<AccelerationStructure>,
    _positions: Vec<Buffer>,
    _instances: Buffer,
}
impl TriangleScene {
    /// Declare the BLAS objects indirectly referenced by the TLAS resident for this encoder.
    pub fn use_resources(&self, encoder: &ComputeCommandEncoderRef) {
        for object in &self.bottom_levels {
            let resource: Resource = object.clone().into();
            encoder.use_resource(&resource, MTLResourceUsage::Read);
        }
    }
}
fn build(
    device: &Device,
    descriptor: &AccelerationStructureDescriptorRef,
) -> Result<AccelerationStructure> {
    let native = device.metal_device();
    let sizes = native.acceleration_structure_sizes_with_descriptor(descriptor);
    if sizes.acceleration_structure_size == 0 {
        return Err(backend("Metal returned zero acceleration structure size"));
    }
    let acceleration =
        native.new_acceleration_structure_with_size(sizes.acceleration_structure_size);
    let scratch = native.new_buffer(
        sizes.build_scratch_buffer_size.max(1),
        MTLResourceOptions::StorageModePrivate,
    );
    let command = device.metal_queue().new_command_buffer();
    let encoder = command.new_acceleration_structure_command_encoder();
    encoder.build_acceleration_structure(&acceleration, descriptor, &scratch, 0);
    encoder.end_encoding();
    complete(command)?;
    Ok(acceleration)
}

pub fn build_triangle_scene(
    device: &Device,
    meshes: &[&[[f32; 3]]],
    instances: &[TriangleInstance],
) -> Result<TriangleScene> {
    if !device.metal_device().supports_raytracing() {
        return Err(GraphicsError::Unsupported);
    }
    if meshes.is_empty()
        || instances.is_empty()
        || meshes.len() > u32::MAX as usize
        || instances.len() > u32::MAX as usize
    {
        return Err(GraphicsError::InvalidArgument);
    }
    // Validate every input before submitting a build or reading a caller-provided slice on GPU.
    for mesh in meshes {
        if mesh.is_empty() || mesh.len() % 3 != 0 || mesh.iter().flatten().any(|v| !v.is_finite()) {
            return Err(GraphicsError::InvalidArgument);
        }
        if std::mem::size_of_val(*mesh) as u64 > device.metal_device().max_buffer_length() {
            return Err(GraphicsError::InvalidArgument);
        }
    }
    for instance in instances {
        let t = instance.transform;
        let determinant = t[0][0] * (t[1][1] * t[2][2] - t[1][2] * t[2][1])
            - t[1][0] * (t[0][1] * t[2][2] - t[0][2] * t[2][1])
            + t[2][0] * (t[0][1] * t[1][2] - t[0][2] * t[1][1]);
        if instance.mesh as usize >= meshes.len()
            || t.iter().flatten().any(|v| !v.is_finite())
            || !determinant.is_finite()
            || determinant == 0.
        {
            return Err(GraphicsError::InvalidArgument);
        }
    }
    objc::rc::autoreleasepool(|| {
        let native = device.metal_device();
        let mut positions = Vec::new();
        let mut bottom_levels = Vec::new();
        for mesh in meshes {
            let buffer = native.new_buffer_with_data(
                mesh.as_ptr().cast(),
                std::mem::size_of_val(*mesh) as u64,
                MTLResourceOptions::StorageModeShared,
            );
            let geometry = AccelerationStructureTriangleGeometryDescriptor::descriptor();
            geometry.set_vertex_buffer(Some(&buffer));
            geometry.set_vertex_stride(12);
            geometry.set_triangle_count((mesh.len() / 3) as u64);
            geometry.set_opaque(true);
            let geometry: AccelerationStructureGeometryDescriptor = geometry.into();
            let descriptor = PrimitiveAccelerationStructureDescriptor::descriptor();
            descriptor.set_geometry_descriptors(Array::from_owned_slice(&[geometry]));
            let descriptor: AccelerationStructureDescriptor = descriptor.into();
            bottom_levels.push(build(device, &descriptor)?);
            positions.push(buffer);
        }
        let descriptors = instances
            .iter()
            .map(|i| MTLAccelerationStructureInstanceDescriptor {
                transformation_matrix: i.transform,
                options: MTLAccelerationStructureInstanceOptions::Opaque
                    | if i.disable_culling {
                        MTLAccelerationStructureInstanceOptions::DisableTriangleCulling
                    } else {
                        MTLAccelerationStructureInstanceOptions::None
                    },
                mask: i.mask,
                intersection_function_table_offset: 0,
                acceleration_structure_index: i.mesh,
            })
            .collect::<Vec<_>>();
        let instance_buffer = native.new_buffer_with_data(
            descriptors.as_ptr().cast(),
            std::mem::size_of_val(descriptors.as_slice()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let descriptor = InstanceAccelerationStructureDescriptor::descriptor();
        descriptor.set_instanced_acceleration_structures(Array::from_owned_slice(&bottom_levels));
        descriptor.set_instance_count(instances.len() as u64);
        descriptor.set_instance_descriptor_buffer(&instance_buffer);
        let descriptor: AccelerationStructureDescriptor = descriptor.into();
        Ok(TriangleScene {
            acceleration: build(device, &descriptor)?,
            bottom_levels,
            _positions: positions,
            _instances: instance_buffer,
        })
    })
}
