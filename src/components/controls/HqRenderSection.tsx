import type { GlossConfig, HqRenderConfig } from "../../types/domain";
import { HqPanelShape } from "../../types/domain";
import { ColorField } from "./ColorField";
import { RangeField } from "./RangeField";
import { SegmentedControl } from "./SegmentedControl";

export interface HqRenderSectionProps {
  value: HqRenderConfig;
  gloss: GlossConfig;
  onChange(patch: Partial<HqRenderConfig>): void;
  onGlossChange(patch: Partial<GlossConfig>): void;
}

const SHAPE_OPTIONS = [
  { value: HqPanelShape.Rect, label: "圆角矩形" },
  { value: HqPanelShape.Squircle, label: "超椭圆" },
  { value: HqPanelShape.Circle, label: "圆形" },
];

const SHADOW_MODE_OPTIONS = [
  { value: "icon" as const, label: "图标轮廓" },
  { value: "badge" as const, label: "面板整体" },
];

export function HqRenderSection(props: HqRenderSectionProps): JSX.Element {
  const { value, onChange, onGlossChange, gloss } = props;

  return (
    <section className="space-y-3">
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        使用图标颜色自动生成面板（还原 MyDockFinder 全局遮罩效果）。此参数区仅对「高质量渲染」预设生效，切换到其他预设后自动失效。
      </p>
      <div className="space-y-3">
          <RangeField
            id="hq-thresh"
            label="明暗分界阈值"
            value={value.thresh}
            min={0}
            max={1}
            step={0.01}
            onChange={(thresh) => onChange({ thresh })}
          />
          <RangeField
            id="hq-icon-ratio"
            label="图标占比"
            value={value.iconRatio}
            min={0.1}
            max={1}
            step={0.01}
            onChange={(iconRatio) => onChange({ iconRatio })}
          />
          <RangeField
            id="hq-bg"
            label="径向光晕浓度"
            value={value.bg}
            min={0}
            max={1}
            step={0.01}
            onChange={(bg) => onChange({ bg })}
          />
          <RangeField
            id="hq-light-mix"
            label="浅色提白比例"
            value={value.lightMix}
            min={0}
            max={1}
            step={0.01}
            onChange={(lightMix) => onChange({ lightMix })}
          />
          <RangeField
            id="hq-dark-mix"
            label="深色加深系数"
            value={value.darkMix}
            min={0}
            max={1}
            step={0.01}
            onChange={(darkMix) => onChange({ darkMix })}
          />
          <RangeField
            id="hq-gloss"
            label="顶部光泽强度"
            value={value.gloss}
            min={0}
            max={1}
            step={0.01}
            onChange={(gloss) => onChange({ gloss })}
          />
          <RangeField
            id="hq-icon-light"
            label="图标提亮"
            value={value.iconLight}
            min={0}
            max={0.5}
            step={0.01}
            onChange={(iconLight) => onChange({ iconLight })}
          />
          <RangeField
            id="hq-corner"
            label="圆角比例"
            value={value.corner}
            min={0.05}
            max={0.5}
            step={0.01}
            onChange={(corner) => onChange({ corner })}
          />
          <div className="grid grid-cols-2 gap-2">
            <RangeField
              id="hq-offset-x"
              label="位置 X"
              value={value.offsetX}
              min={-64}
              max={64}
              step={1}
              onChange={(offsetX) => onChange({ offsetX })}
            />
            <RangeField
              id="hq-offset-y"
              label="位置 Y"
              value={value.offsetY}
              min={-64}
              max={64}
              step={1}
              onChange={(offsetY) => onChange({ offsetY })}
            />
          </div>
          <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
            在 256 画布内平移图标本体（含阴影与光晕），用于调整图标在面板内的位置。
          </p>
          <div className="space-y-1">
            <span className="text-xs text-[var(--text-secondary)]">面板形状</span>
            <SegmentedControl
              id="hq-shape"
              value={value.shape}
              options={SHAPE_OPTIONS}
              onChange={(shape) => onChange({ shape })}
            />
          </div>
          <RangeField
            id="hq-shadow-opacity"
            label="阴影浓度"
            value={value.shadowOpacity}
            min={0}
            max={1}
            step={0.01}
            onChange={(shadowOpacity) => onChange({ shadowOpacity })}
          />
          <RangeField
            id="hq-shadow-blur"
            label="阴影模糊 (×画布)"
            value={value.shadowBlurFactor}
            min={0}
            max={0.1}
            step={0.001}
            onChange={(shadowBlurFactor) => onChange({ shadowBlurFactor })}
          />
          <RangeField
            id="hq-shadow-offset"
            label="阴影偏移 (×画布)"
            value={value.shadowOffsetFactor}
            min={0}
            max={0.1}
            step={0.001}
            onChange={(shadowOffsetFactor) => onChange({ shadowOffsetFactor })}
          />
          <RangeField
            id="hq-shadow-fade"
            label="阴影尾部衰减"
            value={value.shadowFade}
            min={0}
            max={1}
            step={0.01}
            onChange={(shadowFade) => onChange({ shadowFade })}
          />
          <div className="space-y-1">
            <span className="text-xs text-[var(--text-secondary)]">阴影轮廓来源</span>
            <SegmentedControl
              id="hq-shadow-mode"
              value={value.shadowMode}
              options={SHADOW_MODE_OPTIONS}
              onChange={(shadowMode) => onChange({ shadowMode })}
            />
          </div>
          <div className="space-y-3 rounded-lg border border-[var(--border-hairline)] p-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-[var(--text-primary)]">边缘光泽</span>
              <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
                <input
                  type="checkbox"
                  checked={gloss.enabled}
                  onChange={(e) => onGlossChange({ enabled: e.target.checked })}
                  className="rounded"
                />
                启用
              </label>
            </div>
            <div className={gloss.enabled ? "space-y-3" : "pointer-events-none space-y-3 opacity-40"}>
              <RangeField
                id="hq-edge-gloss-width"
                label="边缘宽度"
                value={gloss.width}
                min={1}
                max={16}
                step={0.5}
                onChange={(width) => onGlossChange({ width })}
              />
              <RangeField
                id="hq-edge-gloss-strength"
                label="光泽强度"
                value={gloss.strength}
                min={0}
                max={1}
                step={0.05}
                onChange={(strength) => onGlossChange({ strength })}
              />
              <RangeField
                id="hq-edge-gloss-feather"
                label="内边缘羽化"
                value={gloss.featherBlur}
                min={0}
                max={32}
                step={1}
                onChange={(featherBlur) => onGlossChange({ featherBlur })}
              />
              <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
                羽化值使光泽向内平滑过渡，加强光泽与图标主体的过渡。
              </p>
              <ColorField
                id="hq-edge-gloss-light"
                label="左上高光"
                value={gloss.lightColor}
                pickTarget="glossLight"
                onChange={(lightColor) => onGlossChange({ lightColor })}
              />
              <ColorField
                id="hq-edge-gloss-dark"
                label="右下暗边"
                value={gloss.darkColor}
                pickTarget="glossDark"
                onChange={(darkColor) => onGlossChange({ darkColor })}
              />
            </div>
            <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
              方向性内倒角：左上高光、右下暗边，叠加在自适应面板之上。
            </p>
          </div>
        </div>
    </section>
  );
}