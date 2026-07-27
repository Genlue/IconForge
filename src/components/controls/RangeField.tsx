import { useCallback } from "react";
import { clamp, parseFiniteNumber } from "../../lib/number";

export interface RangeFieldProps {
  id: string;
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit?: string;
  disabled?: boolean;
  onChange(value: number): void;
}

export function RangeField(props: RangeFieldProps): JSX.Element {
  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      props.onChange(clamp(Number(e.target.value), props.min, props.max));
    },
    [props.onChange, props.min, props.max],
  );

  const handleNumberInput = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const n = parseFiniteNumber(e.target.value, props.value);
      props.onChange(clamp(n, props.min, props.max));
    },
    [props.onChange, props.min, props.max, props.value],
  );

  return (
    <div className={`space-y-1 ${props.disabled ? "opacity-40" : ""}`}>
      <label
        htmlFor={props.id}
        className="flex justify-between text-xs text-[var(--text-secondary)]"
      >
        <span>{props.label}</span>
        <span>
          {props.value}
          {props.unit ?? ""}
        </span>
      </label>
      <div className="flex items-center gap-2">
        <input
          id={props.id}
          type="range"
          min={props.min}
          max={props.max}
          step={props.step}
          value={props.value}
          disabled={props.disabled}
          onChange={handleChange}
          className="flex-1"
        />
        <input
          type="number"
          min={props.min}
          max={props.max}
          step={props.step}
          value={props.value}
          disabled={props.disabled}
          onChange={handleNumberInput}
          className="w-14 rounded-md border border-[var(--border-hairline)] bg-white/40 px-1.5 py-0.5 text-xs text-[var(--text-primary)]"
        />
      </div>
    </div>
  );
}
