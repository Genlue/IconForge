pub mod brush;
pub mod color;
pub mod composite;
pub mod mask;
pub mod pipeline;
pub mod resize;
pub mod shadow;
pub mod stroke;
pub mod transform;

pub const MASTER_SIZE: u32 = 256;
// Keep the largest frame first for image viewers, and include the exact
// Windows shell sizes used at common display scaling levels.
pub const ICO_SIZES: [u32; 8] = [256, 128, 64, 48, 32, 24, 20, 16];

pub use brush::composite_brush_strokes;
pub use pipeline::{
    render_icon_set, render_icon_set_with_brushes, render_master, validate_config,
    ValidatedRenderConfig,
};
