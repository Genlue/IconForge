pub mod color;
pub mod composite;
pub mod mask;
pub mod pipeline;
pub mod resize;
pub mod shadow;
pub mod stroke;
pub mod transform;

pub const MASTER_SIZE: u32 = 256;
pub const BASE_SHAPE_INSET: f32 = 16.0;
pub const ICO_SIZES: [u32; 6] = [256, 128, 64, 48, 32, 16];

pub use pipeline::{render_icon_set, render_master, validate_config, ValidatedRenderConfig};
