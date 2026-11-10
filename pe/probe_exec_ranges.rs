//! UI-free unit harness for the native executor's immutable section-range cache.
const PAGE: usize = 4096;
#[path = "src/exec_ranges32.rs"]
mod exec_ranges;
