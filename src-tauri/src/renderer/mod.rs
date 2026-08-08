pub mod brush;
pub mod color;
pub mod composite;
pub mod cutout;
pub mod gloss;
pub mod hq;
pub mod mask;
pub mod pipeline;
pub mod resize;
pub mod shadow;
pub mod stroke;
pub mod transform;
pub mod wand;

pub const MASTER_SIZE: u32 = 256;
// Keep the largest frame first for image viewers, and include the exact
// Windows shell sizes used at common display scaling levels.
pub const ICO_SIZES: [u32; 8] = [256, 128, 64, 48, 32, 24, 20, 16];

pub use brush::composite_brush_strokes;
pub use pipeline::{
    render_icon_set, render_icon_set_with_brushes, render_master, render_master_with_wands,
    validate_config, ValidatedRenderConfig,
};
