//! IPC 数据模型的唯一实现在 `iconforge-domain`（宿主与 wasm 前端共用）。
//! 这里只做转出，保持 `crate::domain::config::…` 这类既有路径不变。

pub use iconforge_domain::{config, input, request, response};
