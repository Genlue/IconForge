import { useCallback } from "react";
import { parseHexRgba, replaceRgb, replaceAlpha } from "../../lib/color";

export interface ColorFieldProps {
  id: string;
  label: string;
  value: string;
  disabled?: boolean;
  onChange(value: string): void;
}

export function ColorField(props: ColorFieldProps): JSX.Element {
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
    <div className={`space-y-1 ${props.disabled ? "opacity-40" : ""}`}>
      <label
        htmlFor={props.id}
        className="text-xs text-[var(--text-secondary)]"
      >
        {props.label}
      </label>
      <div className="flex items-center gap-2">
        <input
          id={props.id}
          type="color"
          value={rgbHex}
          disabled={props.disabled}
          onChange={handleColorChange}
          className="h-7 w-10 cursor-pointer rounded border border-[var(--border-hairline)]"
        />
        <input
          type="range"
          min={0}
          max={255}
          value={a}
          disabled={props.disabled}
          onChange={handleAlphaChange}
          className="flex-1"
        />
        <span className="w-10 text-right text-xs text-[var(--text-secondary)]">
          {Math.round((a / 255) * 100)}%
        </span>
      </div>
    </div>
  );
}
