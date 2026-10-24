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

/** 检查器滑杆行：标签列固定宽度，滑杆与数值输入右对齐成列。 */
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
    <div className={`field${props.disabled ? " field--disabled" : ""}`}>
      <label htmlFor={props.id} className="field__label" title={props.label}>
        {props.label}
      </label>
      <div className="field__controls">
        <input
          id={props.id}
          type="range"
          min={props.min}
          max={props.max}
          step={props.step}
          value={props.value}
          disabled={props.disabled}
          onChange={handleChange}
        />
        <input
          type="number"
          min={props.min}
          max={props.max}
          step={props.step}
          value={props.value}
          disabled={props.disabled}
          onChange={handleNumberInput}
          aria-label={`${props.label}数值`}
          className="num-input"
        />
      </div>
    </div>
  );
}
