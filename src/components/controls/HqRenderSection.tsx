import type { GlossConfig, HqRenderConfig } from "../../types/domain";
import { HqPanelShape } from "../../types/domain";
import { ColorField } from "./ColorField";
import { GlossSection } from "./GlossSection";
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
    <div className="flex flex-col gap-2.5">
      <p className="hint">
        使用图标颜色自动生成面板（还原 MyDockFinder 全局遮罩效果）。此参数区仅对「高质量渲染」预设生效，切换到其他预设后自动失效。
      </p>
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
      <p className="hint">在 256 画布内平移图标本体（含阴影与光晕），用于调整图标在面板内的位置。</p>

      <div className="field">
        <span className="field__label font-medium text-[var(--text-primary)]">自定义基底颜色</span>
        <div className="field__controls justify-end">
          <input
            type="checkbox"
            checked={value.customBgEnabled}
            aria-label="启用自定义基底颜色"
            onChange={(e) => onChange({ customBgEnabled: e.target.checked })}
          />
        </div>
      </div>
      <div className={`flex flex-col gap-2.5${value.customBgEnabled ? "" : " pointer-events-none opacity-40"}`}>
        <ColorField
          id="hq-custom-bg"
          label="基底颜色"
          value={value.customBgColor}
          showAlpha={false}
          onChange={(customBgColor) =>
            onChange({ customBgColor: `${customBgColor.slice(0, 7)}FF` })
          }
        />
        <p className="hint">
          固定使用该颜色作为面板基底（代替从图标主色自动推导的深浅色），径向光晕仍按图标配色。
        </p>
      </div>

      <div className="field">
        <span className="field__label">面板形状</span>
        <div className="field__controls">
          <SegmentedControl
            id="hq-shape"
            value={value.shape}
            options={SHAPE_OPTIONS}
            onChange={(shape) => onChange({ shape })}
          />
        </div>
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
      <div className="field">
        <span className="field__label">阴影轮廓来源</span>
        <div className="field__controls">
          <SegmentedControl
            id="hq-shadow-mode"
            value={value.shadowMode}
            options={SHADOW_MODE_OPTIONS}
            onChange={(shadowMode) => onChange({ shadowMode })}
          />
        </div>
      </div>

      <div className="flex flex-col gap-2.5">
        <span className="field__label font-medium text-[var(--text-primary)]">边缘光泽</span>
        <GlossSection value={gloss} onChange={onGlossChange} />
      </div>
    </div>
  );
}
