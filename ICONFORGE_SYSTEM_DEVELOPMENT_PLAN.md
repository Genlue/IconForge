# IconForge 系统级开发方案

> 文档状态：可执行基线规格（Implementation Baseline）  
> 目标平台：Windows 10 22H2、Windows 11 及其后续兼容版本，x86_64；第一版不承诺 macOS/Linux 功能可用性  
> 技术栈：Tauri 2、Rust stable、React 18、TypeScript、Vite、TailwindCSS  
> 本文中的“必须”是验收要求，“不得”是禁止实现方式。执行 Agent 不得用等价但未经说明的接口替换本文指定的数据契约、渲染顺序或 Windows 持久化方案。

## 0. 强制架构决策与边界

### 0.1 第一版产品边界

1. IconForge 是 Windows 桌面应用。Rust 后端中所有 Windows 专属代码必须放入 `#[cfg(target_os = "windows")]` 模块；非 Windows 构建必须返回结构化错误 `PlatformUnsupported`，不得静默失败。
2. 用户可以拖入图片、`.exe`、`.dll`、`.lnk`、任意普通文件以及目录。由于产品要求 `InputFileType` 只能包含 `Image`、`Exe`、`Lnk`、`Directory` 四个值，任意“非图片、非快捷方式、非目录”的普通文件统一归入 `Exe` 兼容桶，并通过 Windows Shell 文件关联获取图标。字段名 `Exe` 在此处表示“由 Windows Shell/PE 提供图标的普通文件”，不只表示扩展名为 `.exe` 的文件。
3. 拖入目录时，目录本身生成一个 `Directory` 列表项，同时递归扫描其内部文件并生成子列表项。默认最大递归深度为 32、最大结果数为 10,000；超过限制时停止继续扫描并在响应的 `warnings` 中返回原因。不得跟随目录符号链接或 junction，以防循环。
4. 前端实时预览必须使用 DOM 绝对定位、CSS 变量、CSS filter 和内联 SVG 形状路径实现；不得使用 Canvas、WebGL 或在前端进行最终像素烘焙。
5. 导出和应用操作必须把同一份 `RenderConfig` 发送给 Rust，最终 ICO 只能由 Rust 像素管线生成。前端截图或浏览器渲染结果不得作为导出源。
6. `.lnk` 的读取和修改以 Windows COM 接口 `IShellLinkW` + `IPersistFile` 为唯一权威实现。可以引入 `lnk` crate 作为测试解析器或诊断工具，但不得依赖它完成生产环境的写回，因为不同 crate 对 ExtraData、PropertyStore 和未知块的保真度不一致。
7. 应用到 `.lnk` 后被引用的 `.ico` 必须长期存在。系统临时目录只能存放写入过程中的临时文件；最终文件必须移动到 `app_local_data_dir()/managed-icons/`。不得让 `IconLocation` 指向会由系统或应用自动清理的 `%TEMP%` 文件。
8. 所有输入路径在 Rust 中使用 `PathBuf`/`OsStr`，调用 Win32 API 时使用 UTF-16 宽字符串。不得先转换为 ANSI，也不得用 `to_string_lossy()` 作为系统调用参数。

### 0.2 坐标、颜色与数值约定

- 所有最终渲染都先在 `256 x 256` 主画布完成，再由同一主图降采样到 `128/64/48/32/16`。
- 左上角为 `(0, 0)`，X 向右为正，Y 向下为正。
- 百分比字段使用数值本身，例如 `82` 表示 `82%`，不得传 `0.82`。
- 所有颜色字段使用大写或小写均可的 `#RRGGBBAA` 八位十六进制字符串；解析后统一为非预乘 RGBA，再在线性空间中参与混合。不得接受三位色、六位色、CSS 颜色名或 `rgba(...)`。
- 旋转角度为顺时针角度，范围 `-180.0..=180.0`。
- 配置传输允许 JSON 数字，但 Rust 必须拒绝 `NaN` 和无穷大，并在进入渲染器前完成范围校验。
- Rust 内部图像缓冲统一使用 `image::RgbaImage`，内存排列为逐行、每像素四字节 `R,G,B,A`。

### 0.3 推荐依赖及职责

`Cargo.toml` 使用与 Tauri 2 当前稳定脚手架兼容的最新补丁版本，并提交生成后的 `Cargo.lock`。以下 crate 不得由功能重叠且未经说明的库替换：

| crate | 用途 |
|---|---|
| `tauri = "2"` | Command、窗口、应用路径和状态管理 |
| `tauri-plugin-dialog = "2"` | Rust 侧保存文件/选择目录对话框 |
| `tauri-plugin-log = "2"` | 结构化日志，生产环境禁止记录完整像素或用户文件内容 |
| `serde`, `serde_json` | 前后端 JSON 契约 |
| `thiserror` | 内部错误定义 |
| `image` | PNG/JPEG/WebP/BMP/GIF/TIFF 解码及 RGBA 缓冲 |
| `imageproc` | 高斯模糊和基础图像处理；若其函数无法满足边界条件，使用本项目自有卷积实现 |
| `fast_image_resize` | Lanczos3 多尺寸重采样 |
| `ico` | 标准 ICO 目录和图像条目编码 |
| `lnk` | 仅用于测试中交叉解析 LNK 和诊断输出；生产读取/写回仍以 COM 为准 |
| `walkdir` | 不跟随链接的目录遍历 |
| `rayon` | 批量渲染并行化；Win32 COM 操作不得放入 rayon 线程池 |
| `uuid` | 输入项 ID 和持久化图标文件名 |
| `sha2`, `hex` | 配置与源路径内容寻址哈希 |
| `base64` | 缩略图 PNG 传输 |
| `tempfile` | 同目录临时文件及原子替换 |
| `parking_lot` | 内存缓存锁 |
| `lru` | 有容量和内存预算约束的源图缓存 |
| `windows` | Win32 Shell、COM、GDI、资源读取和通知 API |

`windows` crate 至少开启以下 features；执行 Agent 必须使用所选 `windows` 版本中对应的准确 feature 名并以 `cargo check` 验证，不得因为漏 feature 改用未指定的 FFI：

```toml
windows = { version = "0.61", features = [
  "Win32_Foundation",
  "Win32_Graphics_Gdi",
  "Win32_Storage_FileSystem",
  "Win32_System_Com",
  "Win32_System_LibraryLoader",
  "Win32_System_Memory",
  "Win32_System_SystemServices",
  "Win32_UI_Shell",
  "Win32_UI_Shell_Common",
  "Win32_UI_WindowsAndMessaging",
] }
```

`0.61` 是本文编写时的基线大版本，不是永久锁定值；初始化时若 Tauri 的依赖图要求更高兼容版本，可升级，但 Windows API 模块路径、行为和资源释放规则必须保持本文定义。

前端依赖：`react@18`、`react-dom@18`、`typescript`、`vite`、`tailwindcss`、`@tauri-apps/api@2`、`lucide-react`、`zustand`。不得把完整像素处理库加入前端 bundle。

---

## 1. 项目目录结构

执行 Agent 必须创建下列结构。没有实现内容的占位文件也必须创建，并在对应阶段补全；不得把多个职责合并到一个超大文件。

```text
IconForge/
├─ .editorconfig
├─ .gitignore
├─ README.md
├─ package.json
├─ package-lock.json
├─ tsconfig.json
├─ tsconfig.node.json
├─ vite.config.ts
├─ postcss.config.js
├─ tailwind.config.ts
├─ index.html
├─ src/
│  ├─ main.tsx
│  ├─ App.tsx
│  ├─ app.css
│  ├─ vite-env.d.ts
│  ├─ types/
│  │  ├─ domain.ts
│  │  ├─ commands.ts
│  │  └─ errors.ts
│  ├─ constants/
│  │  ├─ defaults.ts
│  │  └─ presets.ts
│  ├─ lib/
│  │  ├─ tauri.ts
│  │  ├─ color.ts
│  │  ├─ number.ts
│  │  └─ squircle.ts
│  ├─ store/
│  │  └─ useIconForgeStore.ts
│  ├─ hooks/
│  │  ├─ useGlobalFileDrop.ts
│  │  ├─ useKeyboardShortcuts.ts
│  │  └─ useDebouncedPreview.ts
│  ├─ components/
│  │  ├─ shell/
│  │  │  ├─ AppShell.tsx
│  │  │  ├─ TitleBar.tsx
│  │  │  └─ StatusBar.tsx
│  │  ├─ preview/
│  │  │  ├─ PreviewPane.tsx
│  │  │  ├─ IconPreview.tsx
│  │  │  ├─ ShapeSvgDefs.tsx
│  │  │  ├─ Checkerboard.tsx
│  │  │  └─ EmptyDropZone.tsx
│  │  ├─ controls/
│  │  │  ├─ ParameterPanel.tsx
│  │  │  ├─ PresetSelector.tsx
│  │  │  ├─ ForegroundSection.tsx
│  │  │  ├─ ShapeSection.tsx
│  │  │  ├─ BackplateSection.tsx
│  │  │  ├─ ShadowSection.tsx
│  │  │  ├─ StrokeSection.tsx
│  │  │  ├─ RangeField.tsx
│  │  │  ├─ ColorField.tsx
│  │  │  └─ SegmentedControl.tsx
│  │  ├─ batch/
│  │  │  ├─ BatchTray.tsx
│  │  │  ├─ BatchItem.tsx
│  │  │  └─ BatchToolbar.tsx
│  │  ├─ export/
│  │  │  ├─ ExportActions.tsx
│  │  │  └─ ExportResultDialog.tsx
│  │  └─ common/
│  │     ├─ GlassPanel.tsx
│  │     ├─ IconButton.tsx
│  │     ├─ Spinner.tsx
│  │     └─ ErrorBanner.tsx
│  └─ tests/
│     ├─ presets.test.ts
│     ├─ squircle.test.ts
│     └─ store.test.ts
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ Cargo.lock
│  ├─ build.rs
│  ├─ tauri.conf.json
│  ├─ capabilities/
│  │  └─ default.json
│  ├─ icons/
│  │  └─ ... Tauri 应用图标
│  └─ src/
│     ├─ main.rs
│     ├─ lib.rs
│     ├─ state.rs
│     ├─ commands/
│     │  ├─ mod.rs
│     │  ├─ import.rs
│     │  ├─ preview.rs
│     │  ├─ export.rs
│     │  └─ apply.rs
│     ├─ domain/
│     │  ├─ mod.rs
│     │  ├─ config.rs
│     │  ├─ input.rs
│     │  ├─ request.rs
│     │  └─ response.rs
│     ├─ error/
│     │  ├─ mod.rs
│     │  └─ app_error.rs
│     ├─ extractor/
│     │  ├─ mod.rs
│     │  ├─ detect.rs
│     │  ├─ image_file.rs
│     │  ├─ pe_resource.rs
│     │  ├─ shell_icon.rs
│     │  ├─ shortcut.rs
│     │  ├─ directory.rs
│     │  └─ hicon.rs
│     ├─ renderer/
│     │  ├─ mod.rs
│     │  ├─ pipeline.rs
│     │  ├─ color.rs
│     │  ├─ mask.rs
│     │  ├─ transform.rs
│     │  ├─ composite.rs
│     │  ├─ shadow.rs
│     │  ├─ stroke.rs
│     │  └─ resize.rs
│     ├─ output/
│     │  ├─ mod.rs
│     │  ├─ ico_writer.rs
│     │  ├─ managed_icon.rs
│     │  └─ atomic_file.rs
│     ├─ windows/
│     │  ├─ mod.rs
│     │  ├─ com.rs
│     │  ├─ shell_link.rs
│     │  ├─ shell_notify.rs
│     │  ├─ resource.rs
│     │  └─ wide_string.rs
│     ├─ services/
│     │  ├─ mod.rs
│     │  ├─ import_service.rs
│     │  ├─ render_service.rs
│     │  └─ export_service.rs
│     └─ tests/
│        ├─ config_validation.rs
│        ├─ mask_golden.rs
│        ├─ renderer_golden.rs
│        ├─ ico_roundtrip.rs
│        └─ fixtures/
│           ├─ transparent.png
│           ├─ opaque.jpg
│           └─ expected/
└─ docs/
   ├─ ARCHITECTURE.md
   ├─ WINDOWS_API.md
   └─ TEST_PLAN.md
```

### 1.1 文件职责

- `src/types/domain.ts` 与 `src-tauri/src/domain/*.rs` 是唯一业务契约源。字段改动必须同时修改两端和契约测试。
- `src/lib/tauri.ts` 是前端唯一允许直接调用 `invoke` 的文件；组件不得直接导入 `@tauri-apps/api/core`。
- `src/store/useIconForgeStore.ts` 保存全局配置、导入项、当前选中项和异步状态。不得把每个文件各自的渲染配置存入列表项，因为产品要求参数对全部列表项全局生效。
- `extractor` 只负责把来源转换成原始 RGBA，不得读取 `RenderConfig`。
- `renderer` 是无文件系统副作用的纯计算模块。
- `output` 负责 ICO 编码、持久化和原子文件操作，不得直接操作 React/Tauri 窗口。
- `windows` 封装所有 `unsafe` Win32/COM 调用。除该目录和 `extractor/hicon.rs` 外，业务模块不得出现裸 `unsafe`。
- `commands` 只做参数反序列化、调用 service 和把 `AppError` 转成可序列化错误；算法不得写在 command 函数中。

---

## 2. 核心数据结构定义

### 2.1 序列化规则

- JSON 对象字段统一为 `camelCase`。
- 枚举通过字符串传输，值严格使用本文给出的 PascalCase 字符串。
- Rust struct 统一添加 `#[serde(rename_all = "camelCase")]`。
- Rust enum 统一添加 `#[serde(rename_all = "PascalCase")]`。
- 可空值使用 TypeScript `T | null` 和 Rust `Option<T>`；不得通过缺失字段表达 null。
- Command 成功值返回普通响应对象；失败值返回 `CommandError`，前端不得解析 Rust debug string。

### 2.2 前端 TypeScript 定义

`src/types/domain.ts` 必须完整定义：

```ts
export enum InputFileType {
  Image = "Image",
  Exe = "Exe",
  Lnk = "Lnk",
  Directory = "Directory",
}

export enum IconShape {
  Rectangle = "Rectangle",
  RoundedRectangle = "RoundedRectangle",
  Squircle = "Squircle",
}

export enum BackplateType {
  None = "None",
  Solid = "Solid",
  Gradient = "Gradient",
}

export enum ExportMode {
  ExportAsIco = "ExportAsIco",
  ApplyToLnk = "ApplyToLnk",
}

export interface OuterShadowConfig {
  enabled: boolean;
  offsetX: number;
  offsetY: number;
  blurRadius: number;
  spread: number;
  color: string;
}

export interface StrokeConfig {
  width: number;
  color: string;
}

export interface RenderConfig {
  foregroundScalePercent: number;
  foregroundOffsetX: number;
  foregroundOffsetY: number;
  foregroundRotationDegrees: number;
  shape: IconShape;
  cornerRadius: number;
  squircleExponent: number;
  backplateType: BackplateType;
  backplateColor: string;
  gradientStartColor: string;
  gradientEndColor: string;
  gradientAngleDegrees: number;
  outerShadow: OuterShadowConfig;
  stroke: StrokeConfig;
}

export interface InputItem {
  id: string;
  sourcePath: string;
  displayName: string;
  fileType: InputFileType;
  parentDirectoryPath: string | null;
  resolvedTargetPath: string | null;
  sourceWidth: number;
  sourceHeight: number;
  thumbnailPngBase64: string;
  supportedExportModes: ExportMode[];
  warnings: string[];
}

export interface PresetDefinition {
  id: string;
  name: string;
  description: string;
  config: RenderConfig;
}
```

字段语义和范围如下：

| 字段 | 类型/范围 | 精确定义 |
|---|---|---|
| `foregroundScalePercent` | `10..=100` | 原图按“包含于画布”的基础尺寸等比缩放后的百分比 |
| `foregroundOffsetX/Y` | `-128..=128` | 相对 256 主画布中心的像素偏移；前端预览按显示尺寸除以 256 等比换算 |
| `foregroundRotationDegrees` | `-180..=180` | 围绕前景包围盒中心顺时针旋转 |
| `cornerRadius` | `0..=128` | 256 主画布中的像素半径；仅 `RoundedRectangle` 生效 |
| `squircleExponent` | `2.0..=8.0` | 超椭圆指数；仅 `Squircle` 生效，默认 `4.0` |
| `backplateColor` | `#RRGGBBAA` | `Solid` 背板颜色；其他类型仍必须传合法值 |
| `gradientStartColor/EndColor` | `#RRGGBBAA` | `Gradient` 两端颜色 |
| `gradientAngleDegrees` | `0..=360` | `0` 表示从左到右，`90` 表示从上到下 |
| `outerShadow.offsetX/Y` | `-64..=64` | 阴影相对形状的像素偏移 |
| `outerShadow.blurRadius` | `0..=64` | 高斯模糊半径；实现使用 `sigma = max(0.01, blurRadius / 2.0)` |
| `outerShadow.spread` | `0..=32` | 模糊前向外膨胀的像素数 |
| `stroke.width` | `0..=32` | 向形状内部绘制的描边宽度；`0` 表示关闭 |

`sourceWidth/sourceHeight` 表示提取到的原始 RGBA 尺寸。由 Shell `HICON` 得到且无法知道资源原尺寸时，填转换后位图尺寸。`thumbnailPngBase64` 只包含 Base64 字符串，不含 `data:image/png;base64,` 前缀；前端显示时自行拼接前缀。

### 2.3 Rust 定义

`src-tauri/src/domain/config.rs`：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IconShape {
    Rectangle,
    RoundedRectangle,
    Squircle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BackplateType {
    None,
    Solid,
    Gradient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OuterShadowConfig {
    pub enabled: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread: f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrokeConfig {
    pub width: f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderConfig {
    pub foreground_scale_percent: f32,
    pub foreground_offset_x: f32,
    pub foreground_offset_y: f32,
    pub foreground_rotation_degrees: f32,
    pub shape: IconShape,
    pub corner_radius: f32,
    pub squircle_exponent: f32,
    pub backplate_type: BackplateType,
    pub backplate_color: String,
    pub gradient_start_color: String,
    pub gradient_end_color: String,
    pub gradient_angle_degrees: f32,
    pub outer_shadow: OuterShadowConfig,
    pub stroke: StrokeConfig,
}
```

`src-tauri/src/domain/input.rs`：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum InputFileType {
    Image,
    Exe,
    Lnk,
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExportMode {
    ExportAsIco,
    ApplyToLnk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputItem {
    pub id: String,
    pub source_path: String,
    pub display_name: String,
    pub file_type: InputFileType,
    pub parent_directory_path: Option<String>,
    pub resolved_target_path: Option<String>,
    pub source_width: u32,
    pub source_height: u32,
    pub thumbnail_png_base64: String,
    pub supported_export_modes: Vec<ExportMode>,
    pub warnings: Vec<String>,
}
```

### 2.4 Command 请求与响应

`src/types/commands.ts` 和 `src-tauri/src/domain/request.rs`/`response.rs` 必须镜像下列契约：

```ts
export interface ImportPathsRequest {
  paths: string[];
}

export interface ImportPathsResponse {
  items: InputItem[];
  rejectedPaths: RejectedPath[];
  warnings: string[];
}

export interface RejectedPath {
  path: string;
  code: string;
  message: string;
}

export interface RenderPreviewRequest {
  sourcePath: string;
  renderConfig: RenderConfig;
  previewSize: number;
}

export interface RenderPreviewResponse {
  pngBase64: string;
  width: number;
  height: number;
}

export interface ExportIcoRequest {
  sourcePaths: string[];
  renderConfig: RenderConfig;
  exportMode: ExportMode.ExportAsIco;
}

export interface ExportedFile {
  sourcePath: string;
  outputPath: string;
}

export interface ExportIcoResponse {
  cancelled: boolean;
  files: ExportedFile[];
  warnings: string[];
}

export interface ApplyToLnkRequest {
  lnkPaths: string[];
  renderConfig: RenderConfig;
  exportMode: ExportMode.ApplyToLnk;
}

export interface AppliedShortcut {
  lnkPath: string;
  managedIconPath: string;
  backupPath: string | null;
}

export interface ApplyToLnkResponse {
  applied: AppliedShortcut[];
  failed: RejectedPath[];
}

export interface CommandError {
  code:
    | "InvalidArgument"
    | "PathNotFound"
    | "UnsupportedImage"
    | "IconExtractionFailed"
    | "ShortcutReadFailed"
    | "ShortcutWriteFailed"
    | "RenderFailed"
    | "IcoEncodeFailed"
    | "IoFailed"
    | "PlatformUnsupported"
    | "Internal";
  message: string;
  path: string | null;
  details: string | null;
}
```

Rust Command 签名固定为：

```rust
#[tauri::command]
pub async fn import_paths(
    request: ImportPathsRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ImportPathsResponse, CommandError>;

#[tauri::command]
pub async fn render_preview(
    request: RenderPreviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<RenderPreviewResponse, CommandError>;

#[tauri::command]
pub async fn export_ico(
    app: tauri::AppHandle,
    request: ExportIcoRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ExportIcoResponse, CommandError>;

#[tauri::command]
pub async fn apply_to_lnk(
    app: tauri::AppHandle,
    request: ApplyToLnkRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ApplyToLnkResponse, CommandError>;
```

请求中的 `exportMode` 必须显式传给 Rust。`export_ico` 只接受 `ExportAsIco`，`apply_to_lnk` 只接受 `ApplyToLnk`；不匹配时返回 `InvalidArgument`。Command 名决定入口，字段用于审计、类型约束以及防止前端误调用。

Rust 请求类型必须按以下定义实现，不得以 `serde_json::Value` 代替：

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportPathsRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderPreviewRequest {
    pub source_path: String,
    pub render_config: RenderConfig,
    pub preview_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportIcoRequest {
    pub source_paths: Vec<String>,
    pub render_config: RenderConfig,
    pub export_mode: ExportMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyToLnkRequest {
    pub lnk_paths: Vec<String>,
    pub render_config: RenderConfig,
    pub export_mode: ExportMode,
}
```

响应类型逐字段对应同节 TypeScript interface：`Vec<T>` 对应 `T[]`，`Option<String>` 对应 `string | null`。每个响应 struct 必须同时派生 `Debug, Clone, Serialize` 并使用 `#[serde(rename_all = "camelCase")]`；任何响应字段不得通过 `serde_json::json!` 临时拼装。

`src-tauri/src/domain/response.rs` 必须包含完整响应定义：

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectedPath {
    pub path: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPathsResponse {
    pub items: Vec<InputItem>,
    pub rejected_paths: Vec<RejectedPath>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPreviewResponse {
    pub png_base64: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFile {
    pub source_path: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportIcoResponse {
    pub cancelled: bool,
    pub files: Vec<ExportedFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedShortcut {
    pub lnk_path: String,
    pub managed_icon_path: String,
    pub backup_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyToLnkResponse {
    pub applied: Vec<AppliedShortcut>,
    pub failed: Vec<RejectedPath>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    pub details: Option<String>,
}
```

`CommandError.code` 在 Rust 中保持 `String` 是为了让 `AppError` 映射简单，但其运行时值必须严格限制为 TypeScript union 中的 11 个字符串；单元测试必须遍历全部 `AppError` 变体验证映射。

`previewSize` 只允许 `64..=1024`，用于生成后端对照预览/测试，不承担拖动滑块时的每帧实时预览。前端滑块交互只更新 CSS；用户停止输入 250ms 后才可调用一次 `render_preview` 做像素级校验，且新请求必须使旧响应失效。

---

## 3. 内置预设方案设计

预设存放于 `src/constants/presets.ts`。预设对象必须深冻结；应用预设时必须深拷贝 `config`，不得让用户编辑反向修改预设常量。

### 3.1 预设 1：macOS 经典圆角

```json
{
  "id": "macos-classic-rounded",
  "name": "macOS 经典圆角",
  "description": "白色圆角背板、轻描边和柔和底部阴影",
  "config": {
    "foregroundScalePercent": 78,
    "foregroundOffsetX": 0,
    "foregroundOffsetY": -2,
    "foregroundRotationDegrees": 0,
    "shape": "RoundedRectangle",
    "cornerRadius": 52,
    "squircleExponent": 4,
    "backplateType": "Solid",
    "backplateColor": "#FFFFFFFF",
    "gradientStartColor": "#FFFFFFFF",
    "gradientEndColor": "#FFFFFFFF",
    "gradientAngleDegrees": 90,
    "outerShadow": {
      "enabled": true,
      "offsetX": 0,
      "offsetY": 10,
      "blurRadius": 22,
      "spread": 0,
      "color": "#0000002E"
    },
    "stroke": {
      "width": 1,
      "color": "#0000001F"
    }
  }
}
```

### 3.2 预设 2：iOS 超椭圆

```json
{
  "id": "ios-squircle",
  "name": "iOS 超椭圆",
  "description": "高饱和渐变超椭圆背板和紧凑前景",
  "config": {
    "foregroundScalePercent": 72,
    "foregroundOffsetX": 0,
    "foregroundOffsetY": 0,
    "foregroundRotationDegrees": 0,
    "shape": "Squircle",
    "cornerRadius": 56,
    "squircleExponent": 4.5,
    "backplateType": "Gradient",
    "backplateColor": "#6E7BFFFF",
    "gradientStartColor": "#8B5CFFFF",
    "gradientEndColor": "#3AA9FFFF",
    "gradientAngleDegrees": 135,
    "outerShadow": {
      "enabled": true,
      "offsetX": 0,
      "offsetY": 12,
      "blurRadius": 26,
      "spread": 1,
      "color": "#1E2A5A38"
    },
    "stroke": {
      "width": 1,
      "color": "#FFFFFF42"
    }
  }
}
```

### 3.3 预设 3：极简纯图

```json
{
  "id": "minimal-glyph",
  "name": "极简纯图",
  "description": "只保留原始图像，不添加背板、阴影或描边",
  "config": {
    "foregroundScalePercent": 100,
    "foregroundOffsetX": 0,
    "foregroundOffsetY": 0,
    "foregroundRotationDegrees": 0,
    "shape": "Rectangle",
    "cornerRadius": 0,
    "squircleExponent": 4,
    "backplateType": "None",
    "backplateColor": "#00000000",
    "gradientStartColor": "#00000000",
    "gradientEndColor": "#00000000",
    "gradientAngleDegrees": 0,
    "outerShadow": {
      "enabled": false,
      "offsetX": 0,
      "offsetY": 0,
      "blurRadius": 0,
      "spread": 0,
      "color": "#00000000"
    },
    "stroke": {
      "width": 0,
      "color": "#00000000"
    }
  }
}
```

### 3.4 默认配置与预设识别

- 首次启动使用 `macos-classic-rounded`。
- 用户选择预设后，store 中设置 `activePresetId` 为对应 ID。
- 用户手动改变任意配置字段后，立即设置 `activePresetId = null`，表示“自定义”。
- 不通过浮点全对象比较自动恢复预设名称；只有用户显式选择预设才能设置非 null ID。

---

## 4. 前端架构与交互逻辑

### 4.1 页面布局与尺寸约束

`AppShell` 使用三行、两列 CSS Grid：

```css
.app-shell {
  display: grid;
  grid-template-columns: minmax(520px, 1fr) 360px;
  grid-template-rows: 52px minmax(420px, 1fr) 184px;
  grid-template-areas:
    "title title"
    "preview controls"
    "batch controls";
  width: 100vw;
  height: 100vh;
  min-width: 960px;
  min-height: 700px;
  overflow: hidden;
}
```

- `TitleBar` 占据 `title`，高度 52px，使用 Tauri 自定义标题栏拖动区；交互按钮必须设置 `data-tauri-drag-region="false"`。
- `PreviewPane` 占据 `preview`，内边距 32px，主图标预览最大 420px，最小 256px，并随容器等比缩放。
- `ParameterPanel` 跨中、下两行占据 `controls`，宽度固定 360px，可纵向滚动，外部不得出现横向滚动条。
- `BatchTray` 占据 `batch`，高度 184px，列表横向滚动。每个 `BatchItem` 为 132x132px，缩略图 72x72px。
- 窗口默认尺寸 1180x820、最小尺寸 960x700。Tauri 窗口透明背景仅在 Windows 合成器稳定时启用；应用根节点必须有不透明度不低于 0.82 的底色，避免透明窗口可读性问题。

macOS 风格的视觉 token 在 `app.css` 中统一定义：

```css
:root {
  --surface-app: rgba(238, 241, 247, 0.92);
  --surface-glass: rgba(255, 255, 255, 0.62);
  --surface-glass-strong: rgba(255, 255, 255, 0.78);
  --border-hairline: rgba(15, 23, 42, 0.10);
  --text-primary: #172033;
  --text-secondary: #687386;
  --accent: #0a84ff;
  --radius-panel: 24px;
  --radius-control: 12px;
  --shadow-panel: 0 18px 48px rgba(30, 41, 59, 0.12);
}

.glass-panel {
  background: var(--surface-glass);
  border: 1px solid var(--border-hairline);
  border-radius: var(--radius-panel);
  box-shadow: var(--shadow-panel);
  backdrop-filter: blur(24px) saturate(145%);
  -webkit-backdrop-filter: blur(24px) saturate(145%);
}
```

不得直接复制 Apple 商标、系统图标或受保护素材。“macOS 风格”仅指层次、玻璃材质、圆角、排版和留白。

### 4.2 React 组件树

```text
App
└─ AppShell
   ├─ TitleBar
   ├─ PreviewPane
   │  ├─ Checkerboard
   │  ├─ EmptyDropZone                      [无导入项]
   │  └─ IconPreview                        [有选中项]
   │     └─ ShapeSvgDefs
   ├─ ParameterPanel
   │  ├─ PresetSelector
   │  ├─ ForegroundSection
   │  ├─ ShapeSection
   │  ├─ BackplateSection
   │  ├─ ShadowSection
   │  ├─ StrokeSection
   │  └─ ExportActions
   ├─ BatchTray
   │  ├─ BatchToolbar
   │  └─ BatchItem[]
   ├─ StatusBar
   ├─ ErrorBanner
   └─ ExportResultDialog
```

组件仅从 Zustand store 读取所需 slice，避免滑块变化导致整个批量列表重渲染。`BatchItem` 使用 `React.memo`，缩略图 URL 使用 `useMemo`。

### 4.3 全局状态

`src/store/useIconForgeStore.ts` 必须包含以下状态和 action：

```ts
export interface IconForgeState {
  items: InputItem[];
  selectedItemId: string | null;
  renderConfig: RenderConfig;
  activePresetId: string | null;
  isImporting: boolean;
  isExporting: boolean;
  dragActive: boolean;
  error: CommandError | null;
  lastExportResult: ExportIcoResponse | ApplyToLnkResponse | null;

  importPaths(paths: string[]): Promise<void>;
  removeItem(id: string): void;
  clearItems(): void;
  selectItem(id: string): void;
  updateRenderConfig(patch: Partial<RenderConfig>): void;
  updateOuterShadow(patch: Partial<OuterShadowConfig>): void;
  updateStroke(patch: Partial<StrokeConfig>): void;
  applyPreset(presetId: string): void;
  exportAsIco(): Promise<void>;
  applyToSelectedLnks(): Promise<void>;
  setDragActive(active: boolean): void;
  clearError(): void;
}
```

状态规则：

1. `renderConfig` 只有一份，对所有 `items` 生效。
2. 第一次成功导入后，若 `selectedItemId` 为 null，则选中第一个成功项。
3. 删除当前选中项后，优先选择原索引处的下一项；不存在则选择前一项；列表为空则设为 null。
4. 再次导入相同规范化路径时不新增重复项。Windows 路径去重使用 Rust 返回的规范路径，前端比较时仅作不区分大小写比较，不自行调用 `realpath`。
5. 每个异步 action 以 `try/finally` 恢复 loading 状态；CommandError 原样保存，不得丢弃 `code` 和 `path`。

### 4.4 拖拽、导入与列表交互

`useGlobalFileDrop.ts` 在挂载时调用 Tauri 2：

```ts
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const unlisten = await getCurrentWebviewWindow().onDragDropEvent((event) => {
  if (event.payload.type === "enter" || event.payload.type === "over") {
    setDragActive(true);
  } else if (event.payload.type === "leave") {
    setDragActive(false);
  } else if (event.payload.type === "drop") {
    setDragActive(false);
    void importPaths(event.payload.paths);
  }
});
```

卸载 hook 时必须调用 `unlisten()`。同时在 `window` 上拦截浏览器原生 `dragover` 和 `drop` 并 `preventDefault()`，防止 WebView 导航到本地文件。

导入时序：

1. 前端过滤空字符串，但不按扩展名过滤。
2. 单次调用 `invoke("import_paths", { request: { paths } })` 传递整个路径数组。
3. Rust 对每个顶层路径独立处理；一个路径失败不得取消其他路径。
4. 图片直接解码；可执行文件、DLL、普通文件走资源/Shell 提取；LNK 解析目标和图标位置；目录自身提取文件夹图标后递归枚举内部项。
5. Rust 为每个成功项生成最长边 128px 的透明 PNG 缩略图并 Base64 编码。
6. 前端追加成功项并展示 `rejectedPaths` 警告。不得因为部分失败清空已有列表。
7. 点击 `BatchItem` 只改变 `selectedItemId`。参数不随选择改变。

批量列表中 `.lnk` 项显示“可应用”徽标；其他项只显示“可导出”。`ApplyToLnk` 按钮处理列表中全部 `.lnk` 项，不把 `.exe` 的已解析目标误当作可修改快捷方式。

### 4.5 DOM/CSS 实时预览

`IconPreview` 的根节点为正方形 `position: relative; aspect-ratio: 1 / 1; isolation: isolate;`。所有视觉层使用 `position: absolute; inset: 0;`，层次固定如下：

```text
z-index 0  checkerboard（IconPreview 外部）
z-index 1  shadow-layer
z-index 2  backplate-layer
z-index 3  foreground-layer
z-index 4  stroke-layer
z-index 5  loading/error overlay
```

形状边界固定为主画布四周内缩 16px，常量名为 `BASE_SHAPE_INSET = 16`。前端按 `previewPx / 256` 转换该内缩值。Rust 必须使用同一个数值。该内缩为外阴影预留空间，不增加未定义的配置字段。

根节点写入下列 CSS 变量：

```ts
const ratio = previewSizePx / 256;
const style = {
  "--shape-inset": `${16 * ratio}px`,
  "--fg-offset-x": `${config.foregroundOffsetX * ratio}px`,
  "--fg-offset-y": `${config.foregroundOffsetY * ratio}px`,
  "--fg-scale": String(config.foregroundScalePercent / 100),
  "--fg-rotation": `${config.foregroundRotationDegrees}deg`,
  "--shadow-x": `${config.outerShadow.offsetX * ratio}px`,
  "--shadow-y": `${config.outerShadow.offsetY * ratio}px`,
  "--shadow-blur": `${config.outerShadow.blurRadius * ratio}px`,
} as React.CSSProperties;
```

形状实现：

- `Rectangle`：`clip-path: inset(var(--shape-inset))`。
- `RoundedRectangle`：使用 `inset(... round radius)`；半径等比换算，且 clamp 到形状短边的一半。
- `Squircle`：`ShapeSvgDefs` 生成 `<clipPath clipPathUnits="objectBoundingBox">`。`src/lib/squircle.ts` 使用第 5.2.2 节超椭圆公式均匀采样 256 个点并输出闭合 path；不得用固定 `border-radius` 冒充超椭圆。

背板层：

```css
.backplate-layer {
  inset: var(--shape-inset);
  overflow: hidden;
}
```

- `None` 为透明。
- `Solid` 使用 `background: var(--backplate-color)`。
- `Gradient` 使用 `linear-gradient(calc(90deg + var(--gradient-angle)), start, end)`；加 90 度是因为 CSS 的 0deg 指向上，而本规格 0 度定义为左到右。

前景层：

```css
.foreground-layer {
  inset: var(--shape-inset);
  display: grid;
  place-items: center;
  overflow: hidden;
  transform: none;
}

.foreground-layer > img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  transform:
    translate(var(--fg-offset-x), var(--fg-offset-y))
    rotate(var(--fg-rotation))
    scale(var(--fg-scale));
  transform-origin: center;
  image-rendering: auto;
}
```

背板层和前景层必须应用同一 shape clip。描边层使用 SVG path 叠加，`fill="none"`，描边宽度按比例换算，路径向内缩半个描边宽度，避免被画布裁切。

阴影层使用形状的纯色副本，先通过 `transform: translate(...) scale(...)` 近似 spread，再用 `filter: blur(...)`。它只承担交互预览；Rust 使用精确膨胀和高斯模糊，因此 CSS 与最终像素在边缘可能存在 1px 内差异。用户停止滑块输入 250ms 后，`useDebouncedPreview` 可调用后端生成一张对照 PNG；响应携带前端自增 generation，只有 generation 与当前值相等时才更新校验图。

### 4.6 参数面板行为

- 所有范围控件同时提供 range 和 number input，二者绑定同一字段。
- `onInput` 先 clamp 再写 store；失焦时格式化到最多两位小数。
- 颜色控件由 `<input type="color">` 编辑 RGB，并用独立透明度滑块编辑 AA；最终总是生成 `#RRGGBBAA`。
- `shape !== RoundedRectangle` 时禁用但不清空 `cornerRadius`。
- `shape !== Squircle` 时禁用但不清空 `squircleExponent`。
- `backplateType === None` 时禁用背板颜色控件；已有颜色值保留。
- 阴影 `enabled === false` 时禁用阴影子字段但保留数值。
- 描边 `width === 0` 时颜色控件禁用但保留颜色。
- Reset 按钮恢复当前默认预设 `macos-classic-rounded`，不是把所有数字置零。

### 4.7 Tauri 通讯封装

`src/lib/tauri.ts` 固定提供：

```ts
import { invoke } from "@tauri-apps/api/core";

export const commands = {
  importPaths: (request: ImportPathsRequest) =>
    invoke<ImportPathsResponse>("import_paths", { request }),
  renderPreview: (request: RenderPreviewRequest) =>
    invoke<RenderPreviewResponse>("render_preview", { request }),
  exportIco: (request: ExportIcoRequest) =>
    invoke<ExportIcoResponse>("export_ico", { request }),
  applyToLnk: (request: ApplyToLnkRequest) =>
    invoke<ApplyToLnkResponse>("apply_to_lnk", { request }),
};
```

不得传递 `File`、Blob 或二进制像素给 Rust；导入/导出只传绝对路径、配置和模式。Base64 仅用于 Rust 返回的小缩略图或校验预览。

Tauri `capabilities/default.json` 只授予主窗口所需的 core window/event 权限和 dialog 权限。不得开启 shell execute、任意文件系统读写或 HTTP 权限。用户拖入路径的实际读取由 Rust Command 完成并由后端验证。

### 4.8 前端逐文件导出契约

以下签名是前端源码的公开边界。组件可以定义文件内私有 helper，但不得改变导出名、Props 或职责。

```ts
// src/main.tsx
function bootstrap(): void;

// src/App.tsx
export default function App(): JSX.Element;

// src/types/errors.ts
export function isCommandError(value: unknown): value is CommandError;
export function normalizeInvokeError(value: unknown): CommandError;

// src/constants/defaults.ts
export const DEFAULT_RENDER_CONFIG: Readonly<RenderConfig>;
export const BASE_SHAPE_INSET = 16;

// src/constants/presets.ts
export const BUILT_IN_PRESETS: readonly PresetDefinition[];
export function getPresetById(id: string): PresetDefinition | undefined;

// src/lib/color.ts
export function parseHexRgba(value: string): { r: number; g: number; b: number; a: number };
export function formatHexRgba(r: number, g: number, b: number, a: number): string;
export function replaceRgb(value: string, rgbHex: string): string;
export function replaceAlpha(value: string, alpha: number): string;

// src/lib/number.ts
export function clamp(value: number, min: number, max: number): number;
export function parseFiniteNumber(value: string, fallback: number): number;

// src/lib/squircle.ts
export interface NormalizedPoint { x: number; y: number }
export function sampleSquirclePath(exponent: number, sampleCount?: number): NormalizedPoint[];
export function pointsToSvgPath(points: readonly NormalizedPoint[]): string;

// src/lib/tauri.ts
export const commands: {
  importPaths(request: ImportPathsRequest): Promise<ImportPathsResponse>;
  renderPreview(request: RenderPreviewRequest): Promise<RenderPreviewResponse>;
  exportIco(request: ExportIcoRequest): Promise<ExportIcoResponse>;
  applyToLnk(request: ApplyToLnkRequest): Promise<ApplyToLnkResponse>;
};

// src/hooks/useGlobalFileDrop.ts
export function useGlobalFileDrop(): void;

// src/hooks/useKeyboardShortcuts.ts
export function useKeyboardShortcuts(): void;

// src/hooks/useDebouncedPreview.ts
export interface DebouncedPreviewState {
  pngDataUrl: string | null;
  loading: boolean;
  error: CommandError | null;
}
export function useDebouncedPreview(
  sourcePath: string | null,
  config: RenderConfig,
  previewSize: number,
  delayMs?: number,
): DebouncedPreviewState;
```

组件签名：

```ts
// shell
export function AppShell(): JSX.Element;
export function TitleBar(): JSX.Element;
export interface StatusBarProps { itemCount: number; busy: boolean; message: string | null }
export function StatusBar(props: StatusBarProps): JSX.Element;

// preview
export function PreviewPane(): JSX.Element;
export interface IconPreviewProps { item: InputItem; config: RenderConfig; sizePx: number }
export function IconPreview(props: IconPreviewProps): JSX.Element;
export interface ShapeSvgDefsProps { shape: IconShape; cornerRadius: number; squircleExponent: number; idPrefix: string }
export function ShapeSvgDefs(props: ShapeSvgDefsProps): JSX.Element;
export function Checkerboard(): JSX.Element;
export interface EmptyDropZoneProps { dragActive: boolean }
export function EmptyDropZone(props: EmptyDropZoneProps): JSX.Element;

// controls
export function ParameterPanel(): JSX.Element;
export interface PresetSelectorProps { presets: readonly PresetDefinition[]; activePresetId: string | null; onSelect(id: string): void }
export function PresetSelector(props: PresetSelectorProps): JSX.Element;
export interface ForegroundSectionProps { config: RenderConfig; onChange(patch: Partial<RenderConfig>): void }
export function ForegroundSection(props: ForegroundSectionProps): JSX.Element;
export interface ShapeSectionProps { config: RenderConfig; onChange(patch: Partial<RenderConfig>): void }
export function ShapeSection(props: ShapeSectionProps): JSX.Element;
export interface BackplateSectionProps { config: RenderConfig; onChange(patch: Partial<RenderConfig>): void }
export function BackplateSection(props: BackplateSectionProps): JSX.Element;
export interface ShadowSectionProps { value: OuterShadowConfig; onChange(patch: Partial<OuterShadowConfig>): void }
export function ShadowSection(props: ShadowSectionProps): JSX.Element;
export interface StrokeSectionProps { value: StrokeConfig; onChange(patch: Partial<StrokeConfig>): void }
export function StrokeSection(props: StrokeSectionProps): JSX.Element;
export interface RangeFieldProps { id: string; label: string; value: number; min: number; max: number; step: number; unit?: string; disabled?: boolean; onChange(value: number): void }
export function RangeField(props: RangeFieldProps): JSX.Element;
export interface ColorFieldProps { id: string; label: string; value: string; disabled?: boolean; onChange(value: string): void }
export function ColorField(props: ColorFieldProps): JSX.Element;
export interface SegmentOption<T extends string> { value: T; label: string }
export interface SegmentedControlProps<T extends string> { id: string; value: T; options: readonly SegmentOption<T>[]; onChange(value: T): void }
export function SegmentedControl<T extends string>(props: SegmentedControlProps<T>): JSX.Element;

// batch
export function BatchTray(): JSX.Element;
export interface BatchItemProps { item: InputItem; selected: boolean; onSelect(id: string): void; onRemove(id: string): void }
export const BatchItem: React.MemoExoticComponent<(props: BatchItemProps) => JSX.Element>;
export interface BatchToolbarProps { itemCount: number; onClear(): void }
export function BatchToolbar(props: BatchToolbarProps): JSX.Element;

// export
export function ExportActions(): JSX.Element;
export interface ExportResultDialogProps { result: ExportIcoResponse | ApplyToLnkResponse | null; onClose(): void }
export function ExportResultDialog(props: ExportResultDialogProps): JSX.Element | null;

// common
export interface GlassPanelProps extends React.HTMLAttributes<HTMLDivElement> {}
export function GlassPanel(props: GlassPanelProps): JSX.Element;
export interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> { label: string }
export function IconButton(props: IconButtonProps): JSX.Element;
export interface SpinnerProps { label?: string }
export function Spinner(props: SpinnerProps): JSX.Element;
export interface ErrorBannerProps { error: CommandError | null; onDismiss(): void }
export function ErrorBanner(props: ErrorBannerProps): JSX.Element | null;
```

`main.tsx` 的 `bootstrap()` 创建根节点并在 `React.StrictMode` 中渲染 `App`。`app.css`、`vite-env.d.ts` 以及各构建配置文件不导出运行时函数。测试文件仅导入上述公开边界，不从组件文件访问私有 helper。

---

## 5. Rust 后端核心逻辑

### 5.1 总体模块接口

`src-tauri/src/extractor/mod.rs`：

```rust
pub struct ExtractedIcon {
    pub pixels: image::RgbaImage,
    pub resolved_target_path: Option<std::path::PathBuf>,
    pub warnings: Vec<String>,
}

pub struct IconExtractor<'a> {
    com_sta_worker: &'a ComStaWorker,
}

impl<'a> IconExtractor<'a> {
    pub fn new(com_sta_worker: &'a ComStaWorker) -> Self;
    pub fn classify(path: &std::path::Path) -> Result<InputFileType, AppError>;
    pub fn extract(&self, path: &std::path::Path) -> Result<ExtractedIcon, AppError>;
}
```

`src-tauri/src/renderer/mod.rs`：

```rust
pub const MASTER_SIZE: u32 = 256;
pub const BASE_SHAPE_INSET: f32 = 16.0;
pub const ICO_SIZES: [u32; 6] = [256, 128, 64, 48, 32, 16];

pub fn validate_config(config: &RenderConfig) -> Result<ValidatedRenderConfig, AppError>;
pub fn render_master(
    source: &image::RgbaImage,
    config: &ValidatedRenderConfig,
) -> Result<image::RgbaImage, AppError>;
pub fn render_icon_set(
    source: &image::RgbaImage,
    config: &ValidatedRenderConfig,
) -> Result<Vec<(u32, image::RgbaImage)>, AppError>;
```

`src-tauri/src/output/ico_writer.rs`：

```rust
pub fn encode_ico(images: &[(u32, image::RgbaImage)]) -> Result<Vec<u8>, AppError>;
pub fn write_ico_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<(), AppError>;
```

`src-tauri/src/windows/shell_link.rs`：

```rust
pub struct ShortcutInfo {
    pub target_path: Option<std::path::PathBuf>,
    pub icon_location: Option<std::path::PathBuf>,
    pub icon_index: i32,
}

pub fn read_shortcut(path: &std::path::Path) -> Result<ShortcutInfo, AppError>;
pub fn rewrite_shortcut_icon_atomic(
    lnk_path: &std::path::Path,
    icon_path: &std::path::Path,
    icon_index: i32,
) -> Result<Option<std::path::PathBuf>, AppError>;
```

### 5.2 模块 A：图标提取器

#### 5.2.1 类型判定

`detect::classify` 必须按以下顺序执行：

1. 拒绝相对路径和包含 NUL 的路径；调用 `std::fs::symlink_metadata(path)`，不存在返回 `PathNotFound`。
2. 若为目录，返回 `Directory`。目录 symlink/junction 仍可作为顶层项提取 Shell 图标，但递归时不得跟随。
3. 取扩展名并使用 Unicode 不区分大小写比较。
4. 扩展名为 `png/jpg/jpeg/webp/bmp/gif/tif/tiff/ico` 返回 `Image`。GIF 只取第 1 帧；ICO 取像素面积最大、相同时位深最高的条目。
5. 扩展名为 `lnk` 返回 `Lnk`。
6. 其他所有普通文件返回 `Exe`，包括 `.exe`、`.dll`、无扩展名文件、文档和未知格式。

分类只决定提取策略；图片解码失败时不得伪装成 Shell 图标，必须返回 `UnsupportedImage`，从而让用户知道文件内容无效。

#### 5.2.2 图片提取

`image_file::decode_image(path)`：

```rust
pub fn decode_image(path: &Path) -> Result<RgbaImage, AppError> {
    let reader = image::ImageReader::open(path)
        .map_err(AppError::io_for(path))?
        .with_guessed_format()
        .map_err(AppError::unsupported_image_for(path))?;
    let image = reader.decode()
        .map_err(AppError::unsupported_image_for(path))?;
    Ok(image.to_rgba8())
}
```

解码前检查文件大小，默认上限 256 MiB；解码后检查 `width * height <= 100_000_000`，使用 checked multiplication，防止解压炸弹和内存溢出。

#### 5.2.3 EXE/DLL 的 PE 图标资源提取

对 `.exe` 和 `.dll`，优先直接读取 PE 图标资源，以保留 256px PNG 图标。`pe_resource.rs` 的顺序固定如下：

1. 调用 `LoadLibraryExW(path, None, LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE)`。失败则进入 Shell 回退。
2. 使用 `EnumResourceNamesW(module, RT_GROUP_ICON, callback, lparam)` 收集 `RT_GROUP_ICON` 名称；优先资源 ID 最小的组。必须处理整数资源和字符串资源名。
3. 对选定组调用 `FindResourceW`、`SizeofResource`、`LoadResource`、`LockResource`，读取 `GRPICONDIR`。所有结构使用逐字段 little-endian 读取，不得把未对齐字节直接转成 Rust struct 引用。
4. 校验 `idReserved == 0`、`idType == 1`、`idCount > 0`。每个 `GRPICONDIRENTRY` 含宽、高、色深、字节数和 `nID`。
5. 选择规则：先按 `width * height` 降序，再按 `bitCount` 降序，再按资源 ID 升序；宽或高字节为 `0` 表示 256。
6. 使用条目的 `nID` 调用 `FindResourceW(module, MAKEINTRESOURCEW(nID), RT_ICON)` 读取原始 `RT_ICON` 字节。
7. 在内存中构造只含一个条目的标准 ICO：6 字节 ICONDIR + 16 字节 ICONDIRENTRY + 原始 `RT_ICON` 数据。目录条目的 image offset 固定为 22。
8. 使用 `ico::IconDir::read(Cursor::new(bytes))`，选择唯一 entry 并 `decode()` 为 RGBA。若资源是 PNG 压缩或 DIB，均由 `ico` crate 解码。
9. 使用 RAII guard 在所有返回路径调用 `FreeLibrary(module)`。
10. 任一步失败必须记录 warning 并调用第 5.2.5 节 Shell 回退，而不是让整个导入失败；只有 Shell 回退也失败时才返回 `IconExtractionFailed`。

普通非 PE 文件不调用 `LoadLibraryExW`，直接进入 Shell 回退。

#### 5.2.4 LNK 提取

所有 COM ShellLink 操作必须发给 `AppState.com_sta_worker`。`IconExtractor::extract` 遇到 LNK 时调用 worker，不直接调用 COM。该 worker 是应用启动时创建的专用 `std::thread`：线程入口调用 `CoInitializeEx(None, COINIT_APARTMENTTHREADED)`，`S_OK` 和 `S_FALSE` 都视为成功且最终都必须配对一次 `CoUninitialize()`；其他 HRESULT 使 worker 启动失败。线程通过 `std::sync::mpsc` 接收闭包任务，应用退出时关闭 channel 并调用 `CoUninitialize()`。不得在 rayon 线程或任意 Tokio worker 上反复初始化 STA。

读取 `.lnk` 的精确流程：

1. `CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)` 得到 `IShellLinkW`。
2. 将其 `cast::<IPersistFile>()`。
3. `IPersistFile::Load(PCWSTR(lnk_utf16), STGM_READ)`。
4. 先调用 `IShellLinkW::GetIconLocation`，传入长度 32,768 的 `u16` 缓冲区并接收 `icon_index`。
5. 若返回的图标位置非空，使用 `ExpandEnvironmentStringsW` 展开其中 `%VAR%`，解析相对路径时以 `.lnk` 所在目录为基准，然后从该路径按指定索引提取资源。`icon_index >= 0` 表示组图标的零基序号；`icon_index < 0` 表示资源 ID `abs(icon_index)`，必须匹配该整数 `RT_GROUP_ICON` 名称，不得把绝对值当数组下标。
6. 若图标位置为空或提取失败，调用 `IShellLinkW::GetPath`，缓冲区长度 32,768，flag 使用 `SLGP_RAWPATH`；展开环境变量并保存到 `resolved_target_path`。
7. 对目标路径递归调用“图片/PE/Shell”提取策略，但不得再次把目标 `.lnk` 解析超过 8 层；维护规范化路径集合防止快捷方式环。
8. 若 target 为空、不可访问或递归失败，最后对 `.lnk` 文件自身调用 Shell 回退。

不得调用 `IShellLinkW::Resolve`，因为它可能弹出 UI、搜索移动后的目标或产生不可预测阻塞。首版只读取已保存的原始路径。

#### 5.2.5 Windows Shell 图标回退

`shell_icon::extract_shell_icon(path, preferred_size)` 必须请求系统图像列表，而不是只使用 `ExtractIconW` 的小尺寸结果：

1. 对实际存在的路径调用 `SHGetFileInfoW(path, attributes, &mut info, size_of::<SHFILEINFOW>(), SHGFI_SYSICONINDEX)` 获取 `iIcon`；普通文件 `attributes = FILE_ATTRIBUTE_NORMAL`，目录 `attributes = FILE_ATTRIBUTE_DIRECTORY`。只有该调用失败且需要按扩展名推断通用图标时，第二次调用才增加 `SHGFI_USEFILEATTRIBUTES`。
2. 调用 `SHGetImageList(SHIL_JUMBO, &IImageList::IID, out)` 获取 256px 系统图像列表；若失败依次回退 `SHIL_EXTRALARGE` 和 `SHIL_LARGE`。
3. 调用 `IImageList::GetIcon(iIcon, ILD_TRANSPARENT)` 得到 `HICON`。
4. 用第 5.2.6 节转换为 RGBA，并确保通过 RAII guard 调用 `DestroyIcon`。
5. 若系统图像列表路径失败，才调用 `ExtractIconExW(path, icon_index, large_out, small_out, 1)`。只接收返回计数为 1 且 handle 非空的结果；未选用的另一个 `HICON` 也必须 `DestroyIcon`。`ExtractIconW` 的 `HINSTANCE` 风格特殊返回值容易误判，首版不得使用它。

`SHGFI_USEFILEATTRIBUTES` 会返回“按文件类型推断”的图标，无法获得文件自定义图标，因此实际存在文件的第一尝试不得使用该 flag。它只用于路径访问失败后的最后回退。

#### 5.2.6 HICON 转 RGBA

`hicon::hicon_to_rgba(hicon)` 必须：

1. 调用 `GetIconInfo` 获得 `hbmColor`、`hbmMask`，使用 guard 在结束时分别 `DeleteObject`。
2. 对 `hbmColor` 调用 `GetObjectW` 得到宽高。若无 color bitmap，宽取 mask 宽，高取 mask 高的一半。
3. 创建内存 DC：`CreateCompatibleDC(None)`。
4. 构造 `BITMAPINFO`：`biSize` 正确、`biWidth = width`、`biHeight = -height`（负值表示 top-down）、`biPlanes = 1`、`biBitCount = 32`、`biCompression = BI_RGB`。
5. `CreateDIBSection` 得到 32bpp BGRA 缓冲，选入 DC，清零缓冲后调用 `DrawIconEx(..., DI_NORMAL)`。
6. 把 BGRA 转为 RGBA。若存在任意非零 alpha，保留该 alpha 并将 RGB 视为非预乘前先执行 `rgb = min(255, round(rgb * 255 / alpha))`；若所有 alpha 都为零，则读取单色 mask：mask 位为 0 表示不透明，1 表示透明，并据此写 alpha。
7. 恢复 DC 原对象，调用 `DeleteObject(dib)` 和 `DeleteDC(dc)`。每个 GDI handle 都必须由单一 guard 所有，禁止重复释放。

#### 5.2.7 目录递归

`directory::expand_directory(root)` 使用 `WalkDir::new(root).follow_links(false).max_depth(32)`。行为固定：

- 先为根目录创建一个 `Directory` 项，其缩略图来自 Shell 文件夹图标。
- 再按规范化绝对路径的 Unicode 不区分大小写顺序排序内部 entry，保证列表稳定。
- 跳过目录 entry 本身，只把普通文件作为子项；不为每个嵌套目录再创建列表项，避免列表噪音。
- 跳过 reparse point、symlink、隐藏的系统回收站目录 `$RECYCLE.BIN` 和 `System Volume Information`。
- 最多返回 10,000 个内部文件；达到上限加入 warning。
- 某个内部文件失败时加入 `rejectedPaths`，继续其他文件。

### 5.3 模块 B：渲染管线

#### 5.3.1 配置校验

`validate_config` 必须逐字段校验第 2.2 节范围和颜色格式。颜色正则语义为 `^#[0-9A-Fa-f]{8}$`，可手写解析而不引入 regex。所有浮点先调用 `is_finite()`。无效配置返回 `InvalidArgument`，错误详情包含字段名，不得自动 clamp 后继续渲染。

#### 5.3.2 形状数学定义

主形状包围盒为 `[16, 16]` 到 `[240, 240]`，宽高均为 224，中心 `(128, 128)`。

- Rectangle：点在包围盒内则 coverage 为 1，否则为 0。
- RoundedRectangle：令半径 `r = min(cornerRadius, 112)`，使用 rounded-box signed distance：

```text
q = abs(p - center) - (halfSize - r)
distance = length(max(q, 0)) + min(max(q.x, q.y), 0) - r
```

`distance <= 0` 为形状内部。

- Squircle：对包围盒中心化，令 `a = b = 112`，指数 `n = squircleExponent`：

```text
F(x, y) = (abs((x - 128) / a))^n + (abs((y - 128) / b))^n
inside when F(x, y) <= 1
```

为抗锯齿，每个 256px 输出像素使用固定 4x4 子像素采样，采样点位于 `(i + 0.5)/4`，coverage 为 16 个样本中位于内部的比例。不得依赖平台字体/GDI 路径栅格器。

#### 5.3.3 色彩和 Alpha 合成

所有 Porter-Duff 合成使用预乘 alpha：

```text
outA = srcA + dstA * (1 - srcA)
outRGBpremul = srcRGBpremul + dstRGBpremul * (1 - srcA)
```

颜色插值和渐变在 linear sRGB 中进行。转换公式：

```text
srgb <= 0.04045 ? srgb / 12.92 : ((srgb + 0.055) / 1.055)^2.4
linear <= 0.0031308 ? 12.92 * linear : 1.055 * linear^(1/2.4) - 0.055
```

最终写入 `RgbaImage` 前解除预乘并转回 sRGB；alpha 为 0 时 RGB 必须置 0。

#### 5.3.4 背板

- `None`：不写颜色，但形状 mask 仍用于裁剪前景。
- `Solid`：每像素使用 `backplateColor * shapeCoverage`。
- `Gradient`：角度 `theta = degrees * PI / 180`，方向向量 `d=(cos(theta), sin(theta))`。将像素相对中心投影到 d，并把覆盖包围盒四角的最小/最大投影映射到 `t=0..1`，在线性空间插值起止颜色，再乘 mask coverage。

#### 5.3.5 前景仿射变换

原图尺寸为 `(sw, sh)`。基础包含缩放：

```text
shapeSize = 224
fitScale = min(shapeSize / sw, shapeSize / sh)
scale = fitScale * foregroundScalePercent / 100
destinationCenter = (128 + offsetX, 128 + offsetY)
rotation = clockwise degrees
```

对每个目标像素中心使用逆变换采样源图：先减 destinationCenter，再逆时针旋转 `-rotation`，再除以 `scale`，最后加源图中心。源坐标之外为透明。采样必须使用预乘 RGBA 双线性插值，避免透明边缘出现黑边。采样结果再乘 shape coverage，然后以 source-over 合成到背板。

#### 5.3.6 描边

描边向内绘制。先生成原 mask `M`，再用与形状定义相同的几何参数生成内缩 `stroke.width` 后的 mask `Mi`。描边 coverage 为 `clamp(M - Mi, 0, 1)`。颜色为 `stroke.color`，在前景之上 source-over 合成。`width == 0` 时跳过计算。

对于 Squircle，内缩后令 `a' = a - width`、`b' = b - width`，中心和指数不变；若任一半轴小于等于 0，`Mi` 全零。

#### 5.3.7 外阴影与高斯模糊

尽管阴影视觉上位于最底层，算法在主体完成后生成阴影，再把主体合成到阴影之上，从而符合“最后应用外阴影”的计算顺序。

阴影源 alpha：背板非 `None` 时使用形状 mask；背板为 `None` 时使用“裁剪后的前景 + 描边”的联合 alpha。处理步骤：

1. `spread > 0` 时对 alpha 做圆盘结构元素的灰度膨胀；半径为 `ceil(spread)`，边界外视为 0。膨胀后的非零 alpha 仅允许存在于主画布内，画布外阴影自然裁切。
2. 高斯参数 `sigma = max(0.01, blurRadius / 2)`，kernel radius 为 `ceil(3 * sigma)`。
3. 一维 kernel：`k(i)=exp(-(i*i)/(2*sigma*sigma))`，计算后归一化，使总和为 1。
4. 先水平后垂直做可分离卷积，使用 `f32` 累积，边界外 alpha 为 0。`blurRadius == 0` 时跳过卷积。
5. 把结果平移 `(offsetX, offsetY)`；使用双线性采样支持小数偏移。
6. 将 shadow color 的 alpha 乘模糊 alpha，写入透明画布。
7. 把步骤 2-5 已完成的主体 source-over 合成到阴影画布。

不得使用 CSS 阴影参数直接近似后端结果。若使用 `imageproc::filter::gaussian_blur_f32`，必须通过 golden test 验证其 sigma、边界和归一化行为满足上述定义。

#### 5.3.8 完整管线伪代码

```rust
pub fn render_master(source: &RgbaImage, cfg: &ValidatedRenderConfig) -> Result<RgbaImage, AppError> {
    let mask = mask::generate_shape_mask(MASTER_SIZE, cfg);
    let mut body = LinearPremultipliedImage::transparent(MASTER_SIZE, MASTER_SIZE);

    if cfg.backplate_type != BackplateType::None {
        composite::over(&mut body, backplate::render(&mask, cfg));
    }

    let foreground = transform::render_foreground(source, cfg, &mask);
    composite::over(&mut body, foreground);

    if cfg.stroke.width > 0.0 {
        composite::over(&mut body, stroke::render_inner_stroke(&mask, cfg));
    }

    let silhouette = if cfg.backplate_type != BackplateType::None {
        mask.clone()
    } else {
        body.alpha_plane()
    };

    let mut result = if cfg.outer_shadow.enabled {
        shadow::render_shadow(&silhouette, cfg)
    } else {
        LinearPremultipliedImage::transparent(MASTER_SIZE, MASTER_SIZE)
    };
    composite::over(&mut result, body);
    Ok(result.into_srgb_rgba8())
}
```

#### 5.3.9 多尺寸重采样

`render_icon_set` 先且只先调用一次 `render_master` 得到 256x256。输出列表顺序固定为 `[256,128,64,48,32,16]`：

- 256 条目直接 clone master，不重复渲染。
- 其他尺寸使用 `fast_image_resize` 的 Lanczos3 filter 从 256 降采样。
- 重采样前将 RGBA 转为预乘 alpha，重采样后解除预乘，避免透明边缘色污染。
- 不得从上一个较小尺寸继续缩小；每个尺寸都从 256 master 生成。

### 5.4 模块 C：输出与应用器

#### 5.4.1 ICO 二进制编码

ICO 文件必须包含 6 个图像条目：256、128、64、48、32、16，均为 32-bit RGBA。

使用 `ico` crate：

```rust
pub fn encode_ico(images: &[(u32, RgbaImage)]) -> Result<Vec<u8>, AppError> {
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for (size, image) in images {
        let icon = ico::IconImage::from_rgba_data(*size, *size, image.clone().into_raw());
        let entry = ico::IconDirEntry::encode_as_png(&icon)
            .map_err(AppError::ico_encode)?;
        dir.add_entry(entry);
    }
    let mut cursor = std::io::Cursor::new(Vec::new());
    dir.write(&mut cursor).map_err(AppError::ico_encode)?;
    Ok(cursor.into_inner())
}
```

六个条目统一使用 PNG 压缩的 32-bit RGBA；256px 目录宽高字节按 ICO 规范写 `0` 表示 256。Windows Vista 及以上支持 PNG 压缩 ICO，本项目的最低平台为 Windows 10，因此无需自写 DIB/AND mask 编码器。写出后测试必须重新用 `ico::IconDir::read` 打开并验证六个尺寸均存在。若后续明确增加 Windows XP 兼容要求，才单独增加小尺寸 BMP/DIB 编码路径，不得在首版中调用版本不确定的 `ico::IconDirEntry::encode`。

#### 5.4.2 `export_ico` Command

输入为一个或多个 `sourcePaths`：

1. 校验非空，规范化并去重。
2. 校验 `RenderConfig`。
3. 一个源文件：使用 `app.dialog().file().add_filter("Windows Icon", &["ico"]).set_file_name("<sanitized-stem>.ico")` 打开保存对话框。取消则返回 `{ cancelled: true, files: [], warnings: [] }`，不得当错误处理。
4. 多个源文件：使用文件对话框选择目标目录。取消行为同上。每个源生成单独 ICO，文件名为 `<sanitized-stem>.ico`。
5. Windows 保留文件名字符 `< > : " / \\ | ? *` 和控制字符替换为 `_`；末尾点/空格移除；空名称改为 `icon`；保留名 `CON/PRN/AUX/NUL/COM1..COM9/LPT1..LPT9` 前加 `_`。
6. 名称冲突依次尝试 `name.ico`、`name (2).ico`、`name (3).ico`；不得覆盖已有文件，除非单文件保存对话框明确返回了用户选择的已存在路径并由对话框完成覆盖确认。
7. 对每个来源调用 extract -> render_icon_set -> encode_ico。
8. 写入目标同目录的 `NamedTempFile`，调用 `write_all`、`flush`、`sync_all`，再 persist/rename 到最终路径。失败时不得留下半个 ICO。
9. 批量导出中单项失败加入 warnings 并继续；若全部失败，返回第一个结构化错误。

对话框属于主线程/插件回调；图像提取和渲染使用 `tauri::async_runtime::spawn_blocking`。不得在 async Command 线程直接运行高斯卷积。

#### 5.4.3 受管 ICO 的持久化路径

`managed_icon::persist_for_shortcut`：

1. 通过 `app.path().app_local_data_dir()` 获取目录，例如 `%LOCALAPPDATA%/com.iconforge.app/`。
2. 创建 `managed-icons/` 子目录。
3. 哈希输入为：规范化 LNK 路径 UTF-8 表示、源图标像素 SHA-256、规范化 `RenderConfig` JSON；字段按 struct 定义顺序序列化，禁止 HashMap。
4. 最终文件名为 `<sha256-lowercase>.ico`。
5. 若文件已存在，先通过 ICO roundtrip 验证；有效则复用，无效则原子重写。
6. 先写同目录临时文件，再原子 rename。`IconLocation` 只能指向最终受管路径。
7. 不自动清理仍被快捷方式引用的文件。第一版不提供垃圾回收；未来实现清理时必须先维护 shortcut-to-icon 索引。

#### 5.4.4 `.lnk` 原子改写

`apply_to_lnk` 只接受扩展名为 `.lnk` 且实际存在的普通文件。每个快捷方式独立处理：

1. 提取该 LNK 当前显示的源图标。
2. 用全局配置渲染并编码 ICO。
3. 把 ICO 持久化到 `managed-icons/`。
4. 在 `.lnk` 同目录创建唯一临时文件 `<name>.iconforge-<uuid>.tmp.lnk`。创建时使用 `OpenOptions::new().write(true).create_new(true)` 预占文件名，随后删除这个零字节占位文件，再让 `IPersistFile::Save` 创建同一路径；若任一步失败则重新生成 UUID，最多尝试 10 次。
5. 在 COM STA worker 中加载原 LNK：`CoCreateInstance` -> `IPersistFile::Load(original, STGM_READ)`。
6. 调用 `IShellLinkW::SetIconLocation(PCWSTR(managed_icon_utf16), 0)`。
7. 调用 `IPersistFile::Save(temp_lnk, BOOL(0))`，再调用 `IPersistFile::SaveCompleted(temp_lnk)`；不得直接 Save 到原文件。
8. 验证临时 LNK 可再次加载，且 `GetIconLocation` 返回受管 ICO 规范路径和 index 0。
9. 使用 `ReplaceFileW(original, temp, backup, REPLACEFILE_WRITE_THROUGH, None, None)` 原子替换。备份路径默认为同目录 `<original-name>.iconforge.bak`；若已存在，用 `.bak.2` 递增，不得覆盖旧备份。
10. 若 `ReplaceFileW` 因文件系统不支持而返回 `ERROR_UNABLE_TO_MOVE_REPLACEMENT`，回退为先 `CopyFileW(original, backup, fail_if_exists=true)`，再 `MoveFileExW(temp, original, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`。备份成功前不得替换原文件。
11. 任一步失败时删除本次临时 `.tmp.lnk`；保留已生成的受管 ICO是允许的，因为它不破坏用户数据。不得删除原 LNK。
12. 成功后返回备份路径，便于 UI 告知用户恢复位置。

这比“直接解析并重写 LNK 二进制字段”更安全：COM 会保留 ShellLinkHeader、IDList、StringData、ExtraData 和未知扩展块。执行 Agent 不得手写偏移覆盖 `IconLocation`。

#### 5.4.5 Shell 缓存刷新

每个成功替换的快捷方式调用：

```rust
SHChangeNotify(
    SHCNE_UPDATEITEM,
    SHCNF_PATHW | SHCNF_FLUSHNOWAIT,
    Some(lnk_path_pcvoid),
    None,
);
SHChangeNotify(
    SHCNE_UPDATEITEM,
    SHCNF_PATHW | SHCNF_FLUSHNOWAIT,
    Some(icon_path_pcvoid),
    None,
);
```

整个批次完成后调用一次：

```rust
SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST | SHCNF_FLUSHNOWAIT, None, None);
```

`SHCNF_PATHW` 的 item 参数必须是在调用期间持续有效的 NUL 结尾 UTF-16 指针。不得传 Rust `String` 地址。使用内容哈希产生新 ICO 文件名可避免 Shell 把旧图标路径的缓存内容继续复用。

### 5.5 AppState、缓存与并发

```rust
pub struct AppState {
    pub source_cache: parking_lot::RwLock<LruCache<SourceCacheKey, Arc<RgbaImage>>>,
    pub com_sta_worker: ComStaWorker,
}
```

- 缓存 key 至少包含规范路径、文件长度、修改时间纳秒；LNK 还包含其目标/图标位置字符串。
- 缓存最多 256 项或估算像素内存 512 MiB，先达到者触发 LRU 淘汰。
- 批量纯 Rust 渲染最多使用 `min(std::thread::available_parallelism(), 4)` 个并行任务，防止 UI 卡顿和内存峰值。
- 同一输出路径使用进程内 mutex 串行写入。
- Command 不接收前端提供的输出任意路径，除了系统保存对话框返回值；`apply_to_lnk` 仅修改请求中明确列出的 LNK。

### 5.6 Tauri 初始化

`src-tauri/src/lib.rs` 必须注册插件、状态和四个 Command：

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(AppState::new().expect("failed to initialize app state"))
        .invoke_handler(tauri::generate_handler![
            commands::import_paths,
            commands::render_preview,
            commands::export_ico,
            commands::apply_to_lnk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running IconForge");
}
```

`main.rs` 只调用 `iconforge_lib::run()`。Windows release 构建添加 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`，避免控制台窗口。

### 5.7 错误、日志与安全

- `AppError` 内部可包含 source error；转成 `CommandError` 时，`message` 给用户可理解的说明，`details` 仅在 debug 构建含底层错误链。
- 日志记录 command 名、耗时、路径 hash、图片尺寸和错误 code。默认不得记录完整用户路径；debug 模式可记录路径，但不得记录文件内容或 Base64。
- 所有尺寸、偏移、buffer 长度、ICO entry offset 使用 checked arithmetic。
- Win32 返回的 handle 和指针每次使用前检查 null/invalid；每个 `unsafe` 函数前写 `// SAFETY:` 注释说明生命周期、缓冲长度和所有权。
- 不解析或执行拖入文件中的代码；`LoadLibraryExW` 必须带 `LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE`，不得调用普通 `LoadLibraryW`，避免执行 DLL 入口点。
- 不请求管理员权限。受保护目录写入失败时返回 `IoFailed`，由用户选择可写位置或移动快捷方式。

### 5.8 Rust 逐文件函数与类型契约

以下是各 Rust 源文件必须暴露的 `pub` 或 `pub(crate)` 边界；未列出的 helper 必须保持私有。所有返回 `AppError` 的函数都不得 panic。

```rust
// state.rs
impl AppState {
    pub fn new() -> Result<Self, AppError>;
}

// error/app_error.rs
impl AppError {
    pub fn code(&self) -> &'static str;
    pub fn path(&self) -> Option<&std::path::Path>;
    pub fn into_command_error(self) -> CommandError;
}

// extractor/detect.rs
pub fn classify(path: &Path) -> Result<InputFileType, AppError>;
pub fn normalize_existing_path(path: &Path) -> Result<PathBuf, AppError>;

// extractor/image_file.rs
pub fn decode_image(path: &Path) -> Result<RgbaImage, AppError>;

// extractor/pe_resource.rs
pub fn extract_pe_icon(path: &Path, icon_selector: IconSelector) -> Result<RgbaImage, AppError>;
pub enum IconSelector { Largest, Index(u32), ResourceId(u16) }

// extractor/shell_icon.rs
pub fn extract_shell_icon(path: &Path, preferred_size: u32) -> Result<RgbaImage, AppError>;

// extractor/shortcut.rs
pub fn extract_shortcut_icon(path: &Path, max_depth: u8) -> Result<ExtractedIcon, AppError>;

// extractor/directory.rs
#[derive(Debug, Clone, Copy)]
pub struct DirectoryLimits { pub max_depth: usize, pub max_files: usize }
pub struct ExtractedImport {
    pub source_path: PathBuf,
    pub file_type: InputFileType,
    pub icon: ExtractedIcon,
    pub parent_directory_path: Option<PathBuf>,
}
pub struct DirectoryExpansion {
    pub root: ExtractedImport,
    pub children: Vec<Result<ExtractedImport, AppError>>,
    pub warnings: Vec<String>,
}
pub fn expand_directory(path: &Path, limits: DirectoryLimits) -> Result<DirectoryExpansion, AppError>;

// extractor/hicon.rs
pub fn hicon_to_rgba(hicon: HICON) -> Result<RgbaImage, AppError>;

// extractor/mod.rs
impl<'a> IconExtractor<'a> {
    pub fn new(com_sta_worker: &'a ComStaWorker) -> Self;
    pub fn classify(path: &Path) -> Result<InputFileType, AppError>;
    pub fn extract(&self, path: &Path) -> Result<ExtractedIcon, AppError>;
}

// renderer/color.rs
pub struct LinearRgba { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }
pub fn parse_hex_rgba(value: &str) -> Result<LinearRgba, AppError>;
pub fn srgb_channel_to_linear(value: f32) -> f32;
pub fn linear_channel_to_srgb(value: f32) -> f32;

// renderer/mask.rs
pub type AlphaMask = Vec<f32>;
pub fn generate_shape_mask(size: u32, config: &ValidatedRenderConfig) -> AlphaMask;
pub fn generate_inset_shape_mask(size: u32, config: &ValidatedRenderConfig, inset: f32) -> AlphaMask;

// renderer/transform.rs
pub fn render_foreground(source: &RgbaImage, config: &ValidatedRenderConfig, mask: &AlphaMask) -> LinearPremultipliedImage;

// renderer/composite.rs
pub struct LinearPremultipliedImage { /* width, height, Vec<[f32; 4]> 均为私有 */ }
impl LinearPremultipliedImage {
    pub fn transparent(width: u32, height: u32) -> Self;
    pub fn alpha_plane(&self) -> AlphaMask;
    pub fn into_srgb_rgba8(self) -> RgbaImage;
}
pub fn over(destination: &mut LinearPremultipliedImage, source: LinearPremultipliedImage);

// renderer/shadow.rs
pub fn dilate_alpha(mask: &AlphaMask, width: u32, height: u32, radius: f32) -> AlphaMask;
pub fn gaussian_blur_alpha(mask: &AlphaMask, width: u32, height: u32, blur_radius: f32) -> AlphaMask;
pub fn render_shadow(mask: &AlphaMask, config: &ValidatedRenderConfig) -> LinearPremultipliedImage;

// renderer/stroke.rs
pub fn render_inner_stroke(mask: &AlphaMask, config: &ValidatedRenderConfig) -> LinearPremultipliedImage;

// renderer/resize.rs
pub fn resize_from_master(master: &RgbaImage, size: u32) -> Result<RgbaImage, AppError>;

// renderer/pipeline.rs and renderer/mod.rs re-export
pub fn validate_config(config: &RenderConfig) -> Result<ValidatedRenderConfig, AppError>;
pub fn render_master(source: &RgbaImage, config: &ValidatedRenderConfig) -> Result<RgbaImage, AppError>;
pub fn render_icon_set(source: &RgbaImage, config: &ValidatedRenderConfig) -> Result<Vec<(u32, RgbaImage)>, AppError>;

// output/ico_writer.rs
pub fn encode_ico(images: &[(u32, RgbaImage)]) -> Result<Vec<u8>, AppError>;
pub fn write_ico_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError>;

// output/managed_icon.rs
pub fn persist_for_shortcut(app: &AppHandle, lnk_path: &Path, source: &RgbaImage, config: &RenderConfig, ico_bytes: &[u8]) -> Result<PathBuf, AppError>;

// output/atomic_file.rs
pub fn write_bytes_atomic(path: &Path, bytes: &[u8], overwrite: bool) -> Result<(), AppError>;
pub fn next_available_path(directory: &Path, stem: &str, extension: &str) -> Result<PathBuf, AppError>;

// windows/com.rs
impl ComStaWorker {
    pub fn spawn() -> Result<Self, AppError>;
    pub fn execute<T, F>(&self, task: F) -> Result<T, AppError>
    where T: Send + 'static, F: FnOnce() -> Result<T, AppError> + Send + 'static;
    pub fn shutdown(&self) -> Result<(), AppError>;
}

// windows/shell_link.rs
pub fn read_shortcut(path: &Path) -> Result<ShortcutInfo, AppError>;
pub fn rewrite_shortcut_icon_atomic(lnk_path: &Path, icon_path: &Path, icon_index: i32) -> Result<Option<PathBuf>, AppError>;

// windows/shell_notify.rs
pub fn notify_icon_changed(lnk_path: &Path, icon_path: &Path);
pub fn notify_associations_changed();

// windows/resource.rs
pub enum ResourceName { Id(u16), Name(Vec<u16>) }
pub fn enumerate_group_icons(module: HMODULE) -> Result<Vec<ResourceName>, AppError>;
pub fn load_resource_bytes(module: HMODULE, resource_type: PCWSTR, name: &ResourceName) -> Result<Vec<u8>, AppError>;

// windows/wide_string.rs
pub fn to_wide_null(value: &OsStr) -> Vec<u16>;
pub fn from_wide_null_terminated(value: &[u16]) -> OsString;
pub fn expand_environment_path(value: &OsStr) -> Result<PathBuf, AppError>;

// services/import_service.rs
pub fn import_paths(request: ImportPathsRequest, state: &AppState) -> Result<ImportPathsResponse, AppError>;

// services/render_service.rs
pub fn render_preview(request: RenderPreviewRequest, state: &AppState) -> Result<RenderPreviewResponse, AppError>;

// services/export_service.rs
pub fn prepare_ico(source_path: &Path, config: &RenderConfig, state: &AppState) -> Result<Vec<u8>, AppError>;
pub fn export_ico_batch(app: &AppHandle, request: ExportIcoRequest, state: &AppState) -> Result<ExportIcoResponse, AppError>;
pub fn apply_to_shortcuts(app: &AppHandle, request: ApplyToLnkRequest, state: &AppState) -> Result<ApplyToLnkResponse, AppError>;
```

`ValidatedRenderConfig` 定义在 `renderer/pipeline.rs`，必须把所有颜色预解析为 `LinearRgba`，把角度预计算为弧度，并保留已验证的数值字段；其所有字段对 `renderer` 子模块使用 `pub(crate)`，对其他模块保持不透明。`SourceCacheKey` 定义在 `state.rs`，字段为 `normalized_path: PathBuf`、`file_len: u64`、`modified_nanos: u128`、`shortcut_signature: Option<String>`，并派生 `Clone, Debug, Hash, PartialEq, Eq`。

`domain/*.rs` 只定义第 2 节的数据类型及其 `validate`/转换实现；`commands/*.rs` 只定义第 2.4 节四个 Tauri Command。`mod.rs` 只声明子模块和重导出，不实现算法。`main.rs`、`lib.rs` 的函数边界见第 5.6 节。`build.rs` 只调用 `tauri_build::build()`。

`ComStaWorker::shutdown` 必须幂等；`Drop` 中做 best-effort shutdown，但显式应用退出路径仍需调用一次。非 Windows 的 `windows/mod.rs` 提供相同高层签名的 stub 并统一返回 `PlatformUnsupported`，不得尝试定义假 Win32 handle。

---

## 6. 执行指令

执行 Agent 必须严格按以下阶段顺序实施。每一阶段的“退出条件”全部满足后才能进入下一阶段；不得先实现 `.lnk` 写回再补基础渲染测试。

### 阶段 1：初始化 Tauri 2 + React + TypeScript 环境与依赖

任务：

1. 使用 Tauri 2 React TypeScript 模板初始化当前目录，不创建嵌套项目目录。
2. 安装第 0.3 节前后端依赖，提交 lockfiles。
3. 建立第 1 节完整目录和空模块导出关系。
4. 配置 Tailwind 扫描 `index.html` 和 `src/**/*.{ts,tsx}`。
5. 配置 `tauri.conf.json`：productName `IconForge`、identifier `com.iconforge.app`、默认/最小窗口尺寸、自定义标题栏。
6. 配置最小 capabilities；确认没有 shell execute 和宽泛 fs scope。
7. 建立 ESLint/TypeScript strict、`cargo fmt`、`cargo clippy -D warnings`、前端测试和 Rust 测试脚本。

退出条件：

- `npm run build` 成功。
- `cargo check --manifest-path src-tauri/Cargo.toml` 成功。
- Tauri 开发窗口可打开且无控制台错误。

### 阶段 2：搭建前端框架与 macOS 风格 UI

任务：

1. 实现 `AppShell` 三行两列布局和全部玻璃视觉 token。
2. 实现标题栏、预览空状态、右侧参数区骨架、底部批量列表骨架。
3. 实现通用 Range、Color、Segmented 控件及键盘 focus 状态。
4. 实现响应式下限：窗口小于最小尺寸时由系统阻止，不用压缩到不可操作。
5. 加入浅色主题；第一版无需深色主题，但所有颜色必须集中为 CSS 变量。

退出条件：

- 1180x820 和 960x700 下无内容重叠。
- 参数面板可独立滚动，底部列表可横向滚动。
- 键盘 Tab 可到达所有交互控件，focus ring 可见。

### 阶段 3：拖拽混合文件与图标提取通讯

任务：

1. 实现所有 domain/request/response/error 契约和 serde 测试。
2. 实现 Tauri 全局拖放 hook、store 导入 action、批量项增删选中。
3. 依次实现图片、PE 资源、Shell 图标、LNK、目录递归提取器。
4. 实现 HICON 到 RGBA、128px PNG 缩略图和 Base64 返回。
5. 建立 COM STA worker；所有 LNK 读取只通过该 worker。
6. 用 PNG、EXE、DLL、LNK、普通 TXT、目录混合拖入进行手工验证。

退出条件：

- 六类来源均能显示缩略图；部分失败不影响其他项。
- 目录扫描不跟随 junction，达到限制时有 warning。
- 重复路径不产生重复列表项。
- GDI handle 在连续导入 1,000 次后无持续增长；使用 Process Explorer 或测试计数验证。

### 阶段 4：前端 CSS 实时预览与预设

任务：

1. 实现三套 JSON 预设、默认配置和自定义状态。
2. 实现绝对定位的 shadow/backplate/foreground/stroke 层。
3. 实现 Rectangle、RoundedRectangle、由 256 点超椭圆路径构成的 Squircle clip。
4. 实现所有参数控件、范围校验、禁用逻辑和 Reset。
5. 实现 250ms debounce 的后端对照预览接口壳；阶段 5 接入真实 renderer。

退出条件：

- 调整任意参数时当前预览在下一动画帧更新。
- 切换列表项不改变全局参数。
- 三个预设值与第 3 节 JSON 深度相等。
- 前端 squircle 单元测试覆盖指数 2、4、8 和闭合路径。

### 阶段 5：Rust 渲染管线与形状遮罩

任务：

1. 实现配置严格校验、颜色解析和 linear sRGB 转换。
2. 实现 4x4 抗锯齿 Rectangle/RoundedRectangle/Squircle mask。
3. 实现背板纯色/渐变、预乘 alpha 仿射变换和内部描边。
4. 实现 spread 膨胀、可分离高斯模糊和阴影底层合成。
5. 实现 `render_master`、`render_icon_set` 和 `render_preview`。
6. 为三套预设和透明边缘图片生成 golden PNG；把可接受像素差写入测试，单通道最大误差不超过 2，差异像素比例不超过 0.1%。

退出条件：

- Rust 单元测试、golden test 全部通过。
- 透明 PNG 旋转缩放后无黑边。
- 前端对照预览与 CSS 预览的几何偏差肉眼不可见，Rust golden 仍是最终权威。

### 阶段 6：ICO 多尺寸打包导出

任务：

1. 实现从 256 master 独立降采样到五个小尺寸。
2. 实现 256 PNG + 小尺寸 32bpp DIB 的六条目 ICO。
3. 实现单文件保存对话框和批量目录选择对话框。
4. 实现文件名清洗、冲突递增、同目录临时写和原子落盘。
5. 实现导出结果 UI，列出成功路径和 warnings。

退出条件：

- 生成 ICO 可被 Windows Explorer、`ico` crate 和至少一个独立查看器打开。
- Roundtrip 测试确认六个尺寸、RGBA 和透明度存在。
- 取消对话框不显示错误、不创建文件。
- 批量中单项失败不会终止其余项。

### 阶段 7：修改 `.lnk` 快捷方式图标

任务：

1. 实现受管 ICO 内容寻址存储。
2. 实现 COM `SetIconLocation`、临时 LNK 保存、重新加载验证、备份和原子替换。
3. 实现 `SHChangeNotify` 单项与批次通知。
4. UI 只对 LNK 启用应用按钮，展示每项成功/失败和备份路径。
5. 测试带参数、工作目录、描述、热键、显式 IconLocation、环境变量路径和未知 ExtraData 的快捷方式；除 IconLocation 外其余字段必须保持不变。

退出条件：

- Explorer 中快捷方式图标刷新并在应用重启、系统重启后仍有效。
- 删除 `%TEMP%` 不影响已应用图标。
- 原 LNK 的目标、参数、工作目录、描述、热键和窗口显示模式不变。
- 失败注入测试证明 ICO 写入失败、临时 LNK 保存失败、ReplaceFile 失败时原快捷方式仍可用。

### 6.1 最终验收命令

执行 Agent 最终必须运行并记录结果：

```powershell
npm run lint
npm run test -- --run
npm run build
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run tauri build
```

任何一条失败都不得宣称完成。若因代码签名缺失导致安装包签名步骤不可用，必须明确记录为发布阻塞项，但仍须保证未签名本地 build 成功。

### 6.2 必须形成的文档

- `README.md`：安装、开发、构建、支持格式、已知限制。
- `docs/ARCHITECTURE.md`：模块依赖方向和数据流。
- `docs/WINDOWS_API.md`：每个 Win32/COM API、输入参数、返回值检查、handle 释放责任。
- `docs/TEST_PLAN.md`：自动测试、手工测试矩阵和 Windows 版本矩阵。
- 所有 `unsafe` 块必须有相邻 `SAFETY` 说明；公共 Rust 函数和导出的 TypeScript 类型必须有文档注释。

---

## 7. 不得偏离的验收原则

1. CSS 预览负责低延迟交互，Rust renderer 负责最终事实；二者共享数值契约但不得复用前端截图。
2. “任意文件”通过 Windows Shell 关联图标回退实现，不代表每种文件都能解析其内部私有图标格式。
3. PE 资源加载不得执行目标二进制代码。
4. LNK 必须通过 COM 保真改写、先备份后替换；不得手写二进制覆盖。
5. 应用到 LNK 的 ICO 必须是持久化受管文件；临时目录仅用于原子写中间态。
6. 一个批量项失败不得破坏其他项或用户原文件。
7. 所有平台 API handle、COM 初始化、GDI 对象、临时文件和监听器都必须有明确释放路径。
