# Vendored GPUI renderers

Three crates of [GPUI](https://www.gpui.rs) (Zed Industries, Apache-2.0), published as `gpui-pre-*` 0.3.14. Every file is unchanged except for the engraving shader, which draws an image as lines when its corner radii are negative:

| Crate | Changed |
| - | - |
| `gpui-pre-wgpu` | added `src/engraving.wgsl`, one branch in `src/shaders.wgsl`, three includes in `src/wgpu_renderer.rs` |
| `gpui-pre-windows` | two functions and one branch in `src/shaders.hlsl` |
| `gpui-pre-apple` | two functions and one branch in `src/shaders.metal` |
