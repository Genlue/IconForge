import { useState } from "react";
import type { AutoCutoutConfig } from "../../types/domain";
import { RangeField } from "./RangeField";
import { WandSection } from "./WandSection";

export function SourceProcessingSection(props: {
  cutout: AutoCutoutConfig;
  onCutoutChange(patch: Partial<AutoCutoutConfig>): void;
  disabled: boolean;
}): JSX.Element {
  const { cutout, onCutoutChange, disabled } = props;
  const [open, setOpen] = useState(false);

  return (
    <section className="space-y-3">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-center justify-between text-xs font-semibold text-[var(--text-primary)]"
      >
        <span>原图处理</span>
        <span className="text-[var(--text-secondary)]">{open ? "▾" : "▸"}</span>
      </button>
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        以下操作直接作用于导入的原图：自动抠图识别背景，魔棒删除相似颜色区域。
      </p>
      {open && (
        <div className={`space-y-3 ${disabled ? "pointer-events-none opacity-40" : ""}`}>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h4 className="text-[11px] font-semibold text-[var(--text-secondary)]">自动抠图</h4>
              <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
                <input
                  type="checkbox"
                  checked={cutout.enabled}
                  onChange={(e) => onCutoutChange({ enabled: e.target.checked })}
                  className="rounded"
                />
                启用
              </label>
            </div>
            <div className={cutout.enabled ? "space-y-3" : "pointer-events-none space-y-3 opacity-40"}>
              <RangeField id="cutout-tolerance" label="背景容差" value={cutout.tolerance} min={0} max={100} step={1} onChange={(tolerance) => onCutoutChange({ tolerance })} />
              <RangeField id="cutout-feather" label="边缘羽化" value={cutout.feather} min={0} max={32} step={1} onChange={(feather) => onCutoutChange({ feather })} />
            </div>
            <p className="text-[11px] leading-4 text-[var(--text-secondary)]">从图像边界识别连通背景，适合白底、纯色底和近纯色底图标。</p>
          </div>
          <hr className="border-[var(--border-hairline)]" />
          <WandSection />
        </div>
      )}
    </section>
  );
}