//! IconForge 的 IPC 数据模型。
//!
//! 这份 crate 只依赖 `serde`：宿主（`src-tauri`）与前端（`ui`，wasm）都从
//! 这里取类型，所以「前端字段名」与「后端反序列化」不可能各写一份再漂移。
//! 任何一端都不能给它加平台依赖 —— 加了就编不出 wasm。

pub mod config;
pub mod input;
pub mod request;
pub mod response;

pub use config::*;
pub use input::*;
pub use request::*;
pub use response::*;
