export interface SegmentOption<T extends string> {
  value: T;
  label: string;
}

export interface SegmentedControlProps<T extends string> {
  id: string;
  value: T;
  options: readonly SegmentOption<T>[];
  onChange(value: T): void;
}

/** macOS 风格分段控件：灰色凹槽内浮起的活动段。 */
export function SegmentedControl<T extends string>(
  props: SegmentedControlProps<T>,
): JSX.Element {
  return (
    <div className="segmented" role="group">
      {props.options.map((opt) => (
        <button
          key={opt.value}
          type="button"
          aria-pressed={props.value === opt.value}
          onClick={() => props.onChange(opt.value)}
          className={`segmented__segment${
            props.value === opt.value ? " segmented__segment--active" : ""
          }`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
