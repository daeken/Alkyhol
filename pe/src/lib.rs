//! Checked PE64 image layout and a deliberately bounded binary ABI runtime.
//! Input code is trusted native code, not sandboxed. See docs/pe.md.
mod command_line;
pub use command_line::windows_command_line;
mod image;
pub use image::{Image, Import, Section};

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod abi;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod process;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use process::ProcessContext;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod runtime;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use runtime::LoadedImage;

pub type Result<T> = std::result::Result<T, String>;
