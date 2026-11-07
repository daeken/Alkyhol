//! Native source compatibility experiments; not a Windows executable loader.
pub mod com;
pub mod ffi;
pub mod graphics;
pub mod kernel;
pub mod tls;

#[cfg(unix)]
pub mod cpp_host;
#[cfg(unix)]
pub mod cpp_input;
#[cfg(unix)]
pub mod unsupported;
