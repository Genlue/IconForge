# IconForge

IconForge 是一个 Windows 图标处理工具：导入图片、程序、快捷方式和现有图标，实时调整外观并导出多尺寸 ICO。

## 功能

- 导出包含 `256 / 128 / 64 / 48 / 32 / 24 / 20 / 16` 档位的 ICO
- 使用 Real-CUGAN 放大和降噪导入图像
- 调整前景缩放、填充方式、偏移和旋转
- 配置圆角、超椭圆、背板、渐变、阴影、描边与边缘光泽
- 自适应面板渲染（按图标配色生成面板、光晕与阴影）
- 原图编辑：自动抠图、魔棒删除、橡皮擦、饱和度
- 为每个导入图标保存独立配置，或一键应用到全部图标
- 批量导出全部图标，并可将结果应用到 LNK 快捷方式

## 结构

```text
IconForge/
├── ui/                   前端：Leptos 0.8 (CSR) + GlassDesign，编到 wasm32
│   ├── src/lib.rs        bootstrap + 根组件
│   ├── src/state.rs      全部应用状态与动作（唯一的 IPC 发起处）
│   ├── src/components/   外壳 / 来源列表 / 预览 / 原图编辑 / 检查器 / 浮层
│   ├── app.css           自有样式：三栏网格、画布、检查器密度
│   └── build.mjs         cargo → wasm-bindgen → ui/dist
├── crates/domain/        IPC 数据模型（serde-only），宿主与前端共用一份
└── src-tauri/            宿主：渲染、提取、导出、快捷方式写入
```

三者的依赖方向是单向的：`ui` 与 `src-tauri` 都依赖 `iconforge-domain`，谁也不依赖谁。
前端不复制任何渲染逻辑 —— 预览、导出、应用到快捷方式都走同一批 Tauri 命令。

## 开发

```powershell
npm install          # 只装 @tauri-apps/cli
npm run tauri dev    # 会先跑 npm run dev:ui 编 wasm，再起宿主
```

只想确认能编译：

```powershell
npm run check        # wasm 前端 + 宿主各 check 一遍
```

改了 Rust 前端代码要重新编 wasm；`tauri dev` 的 `beforeDevCommand` 已经代为执行，
但热更新不会自动发生，改完重跑一次即可。

## 构建

```powershell
cargo test -p iconforge       # 宿主单元测试
npm run build:ui              # release wasm
npm run tauri build
```

构建产物位于 `target/release/bundle/`。

## 上游：GlassDesign

界面控件与材质来自 [Genlue/GlassDesign](https://github.com/Genlue/GlassDesign)，
以 **git 依赖 + tag** 的方式绑定，不做源码副本：

```toml
# ui/Cargo.toml
glass-design = { git = "https://github.com/Genlue/GlassDesign", tag = "v0.10.0" }

# src-tauri/Cargo.toml
glass-design-host = { git = "https://github.com/Genlue/GlassDesign", tag = "v0.10.0" }
```

升级步骤（顺序不能反）：

1. 读上游 `CHANGELOG.md` 里目标版本段的「消费方要做什么」；
2. 改上面两处 `tag`，`cargo update -p glass-design -p glass-design-host`；
3. `npm run check`，再按 GlassDesign 文档的验收清单过一遍
   （亮/暗两套主题、`Auto / Blur / Solid` 三档、固定档边框盒、焦点与 Esc 栈）；
4. 有 API 变更就改 `ui/src/components/`，不要把上游源码拷进本项目。

仓库里也注册了同名 remote，方便对照上游提交：

```powershell
git remote -v            # glassdesign → https://github.com/Genlue/GlassDesign
git fetch glassdesign --tags
```

**不要**改成 `branch = "main"`：上游主干随时可能编译不过，tag 才是契约版本。
