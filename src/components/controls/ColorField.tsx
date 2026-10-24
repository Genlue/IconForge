import { useCallback } from "react";
import { parseHexRgba, replaceRgb, replaceAlpha } from "../../lib/color";
import type { ColorPickTarget } from "../../types/domain";
import { useIconForgeStore } from "../../store/useIconForgeStore";

export interface ColorFieldProps {
  id: string;
  label: string;
  value: string;
  disabled?: boolean;
  pickTarget?: ColorPickTarget;
  /** Hide the opacity slider (client still stores an #RRGGBBAA value). */
  showAlpha?: boolean;
  onChange(value: string): void;
}

function PipetteIcon(): JSX.Element {
  return (
    <svg
      width="13"
      height="13"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="m2 22 1-1h3l9-9" />
      <path d="M3 21v-3l9-9" />
      <path d="m15 6 3.4-3.4a2.1 2.1 0 1 1 3 3L18 9l.4.4a2.1 2.1 0 1 1-3 3l-3.8-3.8a2.1 2.1 0 1 1 3-3l.4.4Z" />
    </svg>
  );
}

/** 检查器颜色行：标签 | 颜色井 | 取色 | 不透明度。 */
export function ColorField(props: ColorFieldProps): JSX.Element {
  const startColorPicking = useIconForgeStore((s) => s.startColorPicking);
  const activeTarget = useIconForgeStore((s) => s.colorPickTarget);
  const { r, g, b, a } = parseHexRgba(props.value);
  const rgbHex = `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;

  const handleColorChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      props.onChange(replaceRgb(props.value, e.target.value));
    },
    [props.onChange, props.value],
  );

  const handleAlphaChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      props.onChange(replaceAlpha(props.value, Number(e.target.value)));
    },
    [props.onChange, props.value],
  );

  return (
    <div className={`field${props.disabled ? " field--disabled" : ""}`}>
      <label htmlFor={props.id} className="field__label" title={props.label}>
        {props.label}
      </label>
      <div className="field__controls">
        <input
          id={props.id}
          type="color"
          value={rgbHex}
          disabled={props.disabled}
          onChange={handleColorChange}
          className="color-well"
        />
        {props.pickTarget && (
          <button
            type="button"
            disabled={props.disabled}
            onClick={() => startColorPicking(props.pickTarget!)}
            aria-label={`取色：${props.label}`}
            title={`取色：然后在预览图上点击`}
            className={`pipette-btn${activeTarget === props.pickTarget ? " pipette-btn--active" : ""}`}
          >
            <PipetteIcon />
          </button>
        )}
        {props.showAlpha === false ? (
          <span className="field__value">100%</span>
        ) : (
          <>
            <input
              type="range"
              aria-label={`${props.label}不透明度`}
              min={0}
              max={255}
              value={a}
              disabled={props.disabled}
              onChange={handleAlphaChange}
            />
            <span className="field__value">{Math.round((a / 255) * 100)}%</span>
          </>
        )}
      </div>
    </div>
  );
}
