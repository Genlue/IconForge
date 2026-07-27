# IconForge

IconForge 是一个 Windows 图标处理工具，可导入图片、程序、快捷方式和现有图标，实时调整图标外观并导出多尺寸 ICO。

## 功能

- 导出包含 `256 / 128 / 64 / 48 / 32 / 24 / 20 / 16` 档位的 ICO
- 使用 Real-CUGAN 放大和降噪导入图像
- 调整前景缩放、填充方式、偏移和旋转
- 配置圆角、超椭圆、背板、渐变、阴影和描边
- 为每个导入图标保存独立配置，或一键应用到全部图标
- 批量导出全部图标，并可将结果应用到 LNK 快捷方式

## 开发

```powershell
npm install
npm run tauri dev
```

## 构建

```powershell
npm test
cargo test --manifest-path src-tauri\Cargo.toml
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。
