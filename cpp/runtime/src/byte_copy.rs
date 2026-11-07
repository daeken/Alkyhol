//! GPU utility for copies that the macOS buffer blit alignment cannot express.
//! No CPU mapping, rounded ranges, padding writes or alternate shader translator.
use super::*;
use std::sync::OnceLock;
const CHUNK: u64 = 16 * 1024 * 1024;
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Parameters {
    source: u64,
    destination: u64,
    length: u64,
}
struct Chunks {
    next: Parameters,
}
impl Chunks {
    fn new(source: u64, destination: u64, length: u64) -> Result<Self, i32> {
        source.checked_add(length).ok_or(E_INVALIDARG)?;
        destination.checked_add(length).ok_or(E_INVALIDARG)?;
        Ok(Self {
            next: Parameters {
                source,
                destination,
                length,
            },
        })
    }
}
impl Iterator for Chunks {
    type Item = Parameters;
    fn next(&mut self) -> Option<Parameters> {
        if self.next.length == 0 {
            return None;
        }
        let result = Parameters {
            length: self.next.length.min(CHUNK),
            ..self.next
        };
        self.next.source += result.length;
        self.next.destination += result.length;
        self.next.length -= result.length;
        Some(result)
    }
}
#[derive(Default)]
pub(super) struct Kernel {
    pipeline: OnceLock<Result<metal::ComputePipelineState, String>>,
}
impl Kernel {
    pub(super) fn pipeline(
        &self,
        device: &metal::DeviceRef,
    ) -> Result<&metal::ComputePipelineState, i32> {
        match self.pipeline.get_or_init(|| {
            let options = metal::CompileOptions::new();
            let library =
                device.new_library_with_source(include_str!("byte_copy.metal"), &options)?;
            let function = library.get_function("alkyhol_copy_bytes", None)?;
            device.new_compute_pipeline_state_with_function(&function)
        }) {
            Ok(pipeline) => Ok(pipeline),
            Err(error) => {
                eprintln!("Alkyhol byte-copy GPU kernel failed: {error}");
                Err(0x80004005u32 as i32)
            }
        }
    }
}
pub(super) fn aligned(source: u64, destination: u64, length: u64) -> bool {
    source.is_multiple_of(4) && destination.is_multiple_of(4) && length.is_multiple_of(4)
}
// One serial encoder stream spans the batch. Switch encoder kinds in command
// order, relying on tracked, directly bound resources in classic MTLCommandQueue.
pub(super) struct Encoder<'a> {
    command: &'a metal::CommandBufferRef,
    pipeline: Option<&'a metal::ComputePipelineState>,
    blit: Option<&'a metal::BlitCommandEncoderRef>,
    compute: Option<&'a metal::ComputeCommandEncoderRef>,
}
impl<'a> Encoder<'a> {
    pub(super) fn new(
        command: &'a metal::CommandBufferRef,
        pipeline: Option<&'a metal::ComputePipelineState>,
    ) -> Self {
        Self {
            command,
            pipeline,
            blit: None,
            compute: None,
        }
    }
    pub(super) fn copy(
        &mut self,
        source: &metal::BufferRef,
        source_offset: u64,
        destination: &metal::BufferRef,
        destination_offset: u64,
        length: u64,
    ) {
        if length == 0 {
            return;
        }
        if aligned(source_offset, destination_offset, length) {
            if let Some(compute) = self.compute.take() {
                compute.end_encoding();
            }
            let blit = self
                .blit
                .get_or_insert_with(|| self.command.new_blit_command_encoder());
            blit.copy_from_buffer(
                source,
                source_offset,
                destination,
                destination_offset,
                length,
            );
        } else {
            if let Some(blit) = self.blit.take() {
                blit.end_encoding();
            }
            let pipeline = self
                .pipeline
                .expect("byte-copy pipeline preflighted before encoding");
            let compute = self.compute.get_or_insert_with(|| {
                let encoder = self.command.new_compute_command_encoder();
                encoder.set_compute_pipeline_state(pipeline);
                encoder
            });
            compute.set_buffer(0, Some(source), 0);
            compute.set_buffer(1, Some(destination), 0);
            let threads = pipeline.max_total_threads_per_threadgroup().min(256);
            for parameters in Chunks::new(source_offset, destination_offset, length)
                .expect("validated buffer extents")
            {
                compute.set_bytes(
                    2,
                    std::mem::size_of::<Parameters>() as u64,
                    (&parameters as *const Parameters).cast(),
                );
                compute.dispatch_thread_groups(
                    metal::MTLSize::new(parameters.length.div_ceil(threads), 1, 1),
                    metal::MTLSize::new(threads, 1, 1),
                );
            }
        }
    }
}
impl Drop for Encoder<'_> {
    fn drop(&mut self) {
        if let Some(e) = self.blit.take() {
            e.end_encoding();
        }
        if let Some(e) = self.compute.take() {
            e.end_encoding();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parameter_layout_and_chunk_boundaries_preserve_all_bytes_without_overflow() {
        assert_eq!(std::mem::size_of::<Parameters>(), 24);
        assert_eq!(std::mem::offset_of!(Parameters, length), 16);
        assert!(Chunks::new(u64::MAX, 0, 1).is_err());
        assert!(Chunks::new(0, u64::MAX, 1).is_err());
        assert_eq!(Chunks::new(3, 1, 0).unwrap().count(), 0);
        for length in [1, 3, CHUNK - 1, CHUNK, CHUNK + 1, 2 * CHUNK + 3] {
            let rows: Vec<_> = Chunks::new(3, 1, length).unwrap().collect();
            let mut next = Parameters {
                source: 3,
                destination: 1,
                length,
            };
            for row in rows {
                assert_eq!(row.source, next.source);
                assert_eq!(row.destination, next.destination);
                assert!(row.length > 0 && row.length <= CHUNK);
                next.source += row.length;
                next.destination += row.length;
                next.length -= row.length;
            }
            assert_eq!(next.length, 0);
            assert_eq!(next.source, 3 + length);
            assert_eq!(next.destination, 1 + length);
        }
    }
    #[test]
    fn blit_selection_requires_alignment_of_both_offsets_and_length() {
        for source in 0..4 {
            for destination in 0..4 {
                for length in 1..9 {
                    assert_eq!(
                        aligned(source, destination, length),
                        source == 0 && destination == 0 && length % 4 == 0
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;
    #[test]
    fn actual_private_byte_kernel_and_blit_switches_preserve_every_guard_byte() {
        autoreleasepool(|| {
            let device = metal::Device::system_default().expect("Metal device");
            let kernel = Kernel::default();
            let pipeline = kernel.pipeline(&device).unwrap();
            assert!(std::ptr::eq(pipeline, kernel.pipeline(&device).unwrap()));
            assert!(pipeline.max_total_threads_per_threadgroup() > 0);
            let shared = metal::MTLResourceOptions::StorageModeShared
                | metal::MTLResourceOptions::HazardTrackingModeTracked;
            let source = device.new_buffer(1027, shared);
            let destination = device.new_buffer(1027, shared);
            let private = device.new_buffer(
                1027,
                metal::MTLResourceOptions::StorageModePrivate
                    | metal::MTLResourceOptions::HazardTrackingModeTracked,
            );
            unsafe {
                for (i, b) in std::slice::from_raw_parts_mut(source.contents().cast::<u8>(), 1027)
                    .iter_mut()
                    .enumerate()
                {
                    *b = (i * 13 + 7) as u8;
                }
                std::ptr::write_bytes(destination.contents().cast::<u8>(), 0xa5, 1027);
            }
            let queue = device.new_command_queue();
            let command = queue.new_command_buffer();
            {
                let mut encoder = Encoder::new(command, Some(pipeline));
                encoder.copy(&source, 0, &private, 0, 1027);
                encoder.copy(&private, 3, &destination, 1, 17);
                encoder.copy(&private, 32, &destination, 64, 32);
                encoder.copy(&private, 2, &destination, 97, 31);
            }
            command.commit();
            command.wait_until_completed();
            assert_eq!(command.status(), metal::MTLCommandBufferStatus::Completed);
            let actual =
                unsafe { std::slice::from_raw_parts(destination.contents().cast::<u8>(), 1027) };
            for (i, &value) in actual.iter().enumerate() {
                let expected = if (1..18).contains(&i) {
                    ((i + 2) * 13 + 7) as u8
                } else if (64..96).contains(&i) {
                    ((i - 32) * 13 + 7) as u8
                } else if (97..128).contains(&i) {
                    ((i - 95) * 13 + 7) as u8
                } else {
                    0xa5
                };
                assert_eq!(value, expected, "byte {i}");
            }
        });
    }
}
