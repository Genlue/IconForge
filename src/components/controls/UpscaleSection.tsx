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
  { value: "psnr", label: "标准 (PSNR)" },
  { value: "no-denoise", label: "无降噪" },
];

export function UpscaleSection(props: UpscaleSectionProps): JSX.Element {
  const { value, onChange } = props;

  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">画质增强</h3>
      <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
        <input
          type="checkbox"
          checked={value.enabled}
          onChange={(e) => onChange({ enabled: e.target.checked })}
          className="rounded"
        />
        启用 Real-CUGAN 放大
      </label>
      <div className={value.enabled ? "space-y-3" : "pointer-events-none opacity-40 space-y-3"}>
        {/* Scale */}
        <div>
          <label
            htmlFor="upscale-scale"
            className="mb-1 block text-xs text-[var(--text-secondary)]"
          >
            缩放倍数
          </label>
          <select
            id="upscale-scale"
            value={value.scale}
            onChange={(e) => onChange({ scale: Number(e.target.value) })}
            className="w-full rounded border border-[var(--border-hairline)] bg-[var(--bg-secondary)] px-2 py-1 text-xs text-[var(--text-primary)]"
          >
            {SCALE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {/* Denoise */}
        <div>
          <label
            htmlFor="upscale-denoise"
            className="mb-1 block text-xs text-[var(--text-secondary)]"
          >
            降噪级别
          </label>
          <select
            id="upscale-denoise"
            value={value.denoise}
            onChange={(e) => onChange({ denoise: Number(e.target.value) })}
            className="w-full rounded border border-[var(--border-hairline)] bg-[var(--bg-secondary)] px-2 py-1 text-xs text-[var(--text-primary)]"
          >
            {DENOISE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {/* Model */}
        <div>
          <label
            htmlFor="upscale-model"
            className="mb-1 block text-xs text-[var(--text-secondary)]"
          >
            模型
          </label>
          <select
            id="upscale-model"
            value={value.model}
            onChange={(e) => onChange({ model: e.target.value })}
            className="w-full rounded border border-[var(--border-hairline)] bg-[var(--bg-secondary)] px-2 py-1 text-xs text-[var(--text-primary)]"
          >
            {MODEL_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
      </div>
    </section>
  );
}
