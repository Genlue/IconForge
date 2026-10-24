import type { UpscaleConfig } from "../../types/domain";

export interface UpscaleSectionProps {
  value: UpscaleConfig;
  onChange(patch: Partial<UpscaleConfig>): void;
}

const SCALE_OPTIONS = [
  { value: 2, label: "2x" },
  { value: 3, label: "3x" },
  { value: 4, label: "4x" },
];

const DENOISE_OPTIONS = [
  { value: -1, label: "无" },
  { value: 0, label: "低" },
  { value: 1, label: "中" },
  { value: 2, label: "高" },
  { value: 3, label: "极高" },
];

const MODEL_OPTIONS = [
  { value: "se", label: "轻量 (SE)" },
  { value: "pro", label: "高质量 (Pro)" },
  { value: "nose", label: "No-SE（仅 2x 无降噪）" },
];

export function UpscaleSection(props: UpscaleSectionProps): JSX.Element {
  const { value, onChange } = props;

  const updateModel = (model: string) => {
    if (model === "nose") {
      onChange({ model, scale: 2, denoise: -1 });
      return;
    }
    if (model === "pro" && value.scale === 4) {
      onChange({ model, scale: 2, denoise: value.denoise === 1 || value.denoise === 2 ? 3 : value.denoise });
      return;
    }
    onChange({
      model,
      denoise: value.denoise === 1 || value.denoise === 2 ? 3 : value.denoise,
    });
  };

  const updateScale = (scale: number) => {
    if (value.model === "nose") {
      onChange({ scale: 2, denoise: -1 });
      return;
    }
    onChange({
      scale,
      denoise: scale >= 3 && (value.denoise === 1 || value.denoise === 2)
        ? 3
        : value.denoise,
    });
  };

  return (
    <div className="flex flex-col gap-2.5">
      <div className="field">
        <span className="field__label">启用放大</span>
        <div className="field__controls justify-end">
          <input
            type="checkbox"
            checked={value.enabled}
            aria-label="启用 Real-CUGAN 放大"
            onChange={(e) => onChange({ enabled: e.target.checked })}
          />
        </div>
      </div>
      <p className="hint">
        先增强导入源图，再用于最终预览和所有 ICO 档位导出。首次处理可能需要几秒。
      </p>
      <div className={`flex flex-col gap-2.5${value.enabled ? "" : " pointer-events-none opacity-40"}`}>
        <div className="field">
          <label htmlFor="upscale-scale" className="field__label">
            缩放倍数
          </label>
          <div className="field__controls">
            <select
              id="upscale-scale"
              value={value.scale}
              onChange={(e) => updateScale(Number(e.target.value))}
              className="select"
            >
              {SCALE_OPTIONS.map((opt) => (
                <option
                  key={opt.value}
                  value={opt.value}
                  disabled={(value.model === "pro" && opt.value === 4) || (value.model === "nose" && opt.value !== 2)}
                >
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="field">
          <label htmlFor="upscale-denoise" className="field__label">
            降噪级别
          </label>
          <div className="field__controls">
            <select
              id="upscale-denoise"
              value={value.denoise}
              onChange={(e) => onChange({ denoise: Number(e.target.value) })}
              className="select"
            >
              {DENOISE_OPTIONS.map((opt) => (
                <option
                  key={opt.value}
                  value={opt.value}
                  disabled={
                    value.model === "nose"
                      ? opt.value !== -1
                      : value.scale >= 3 && (opt.value === 1 || opt.value === 2)
                  }
                >
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="field">
          <label htmlFor="upscale-model" className="field__label">
            模型
          </label>
          <div className="field__controls">
            <select
              id="upscale-model"
              value={value.model}
              onChange={(e) => updateModel(e.target.value)}
              className="select"
            >
              {MODEL_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>
    </div>
  );
}
