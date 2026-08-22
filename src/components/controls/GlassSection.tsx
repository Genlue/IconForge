import type { GlassRenderConfig } from "../../types/domain";
import { RangeField } from "./RangeField";

export interface GlassSectionProps {
  value: GlassRenderConfig;
  onChange(patch: Partial<GlassRenderConfig>): void;
}

export function GlassSection(props: GlassSectionProps): JSX.Element {
  const { value, onChange } = props;

  return (
    <section className="space-y-3">
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        仅处理源图：转为黑白灰后通过高度场、法线、多光源与边缘反射形成玻璃浮雕。底板、圆角、边缘高光与投影沿用上方经典参数。
      </p>
      <div className="space-y-3 rounded-lg border border-[var(--border-hairline)] p-3">
        <span className="text-xs font-semibold text-[var(--text-primary)]">玻璃浮雕</span>
        <RangeField
          id="glass-bevel"
          label="倒角圆润度"
          value={value.bevelRadius}
          min={0}
          max={0.5}
          step={0.01}
          onChange={(bevelRadius) => onChange({ bevelRadius })}
        />
        <RangeField
          id="glass-normal"
          label="浮雕强度"
          value={value.normalStrength}
          min={0}
          max={2}
          step={0.05}
          onChange={(normalStrength) => onChange({ normalStrength })}
        />
        <RangeField
          id="glass-specular"
          label="高光强度"
          value={value.specularStrength}
          min={0}
          max={1}
          step={0.05}
          onChange={(specularStrength) => onChange({ specularStrength })}
        />
        <RangeField
          id="glass-fresnel"
          label="边缘反射"
          value={value.fresnelStrength}
          min={0}
          max={1}
          step={0.05}
          onChange={(fresnelStrength) => onChange({ fresnelStrength })}
        />
        <RangeField
          id="glass-ao"
          label="环境遮蔽"
          value={value.aoStrength}
          min={0}
          max={1}
          step={0.05}
          onChange={(aoStrength) => onChange({ aoStrength })}
        />
        <RangeField
          id="glass-retention"
          label="色彩保留"
          value={value.colorRetention}
          min={0}
          max={1}
          step={0.01}
          onChange={(colorRetention) => onChange({ colorRetention })}
        />
        <RangeField
          id="glass-contrast"
          label="源图对比度"
          value={value.contrastStrength}
          min={0}
          max={1}
          step={0.05}
          onChange={(contrastStrength) => onChange({ contrastStrength })}
        />
        <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
          原图转为黑白（亮色字形保持白），色彩保留 0 为全灰白，数值越高越接近原色。对比度用于提高源图内明暗层次，让浮雕更清晰。
        </p>
      </div>
    </section>
  );
}