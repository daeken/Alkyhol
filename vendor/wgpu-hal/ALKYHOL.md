# Alkyhol precise occlusion profile

Upstream wgpu-hal 27.0.4, git af91efa93da2af9366b252ed7b9a089175f4874e.
Crates.io archive SHA-256 5b21cb61c57ee198bc4aff71aeadff4cbb80b927beb912506af9c780d64313ce.
Source: https://github.com/gfx-rs/wgpu/tree/af91efa93da2af9366b252ed7b9a089175f4874e/wgpu-hal
Original MIT/Apache-2.0 licenses are retained alongside the original source.

The opt-in Cargo feature `alkyhol-precise-occlusion` changes native occlusion
queries from boolean visibility to actual sample counts: Metal Counting, Vulkan
PRECISE plus its required device feature, and D3D12 OCCLUSION. Other query types
are unchanged. The Vulkan QuerySet retains its type to avoid applying PRECISE to
unrelated query types. Without this feature upstream boolean behavior remains.
The precise Metal profile is desktop macOS only (Counting is part of that Metal
profile). Other Metal targets and Vulkan adapters without precise-count capability
fail device opening with
an explicit log; no boolean result is advertised as a D3D9 sample count.

This is an Alkyhol D3D9 profile, not a claim that upstream WebGPU queries promise
precise counts. Only the d3d9 crate opts into this feature. See docs/validation.md
in the Alkyhol repository for actual backend execution and native comparisons.
