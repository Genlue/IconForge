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

export function SegmentedControl<T extends string>(
  props: SegmentedControlProps<T>,
): JSX.Element {
  return (
    <div className="flex overflow-hidden rounded-lg border border-[var(--border-hairline)] bg-white/40">
      {props.options.map((opt) => (
        <button
          key={opt.value}
          onClick={() => props.onChange(opt.value)}
          className={`min-w-0 flex-1 truncate px-1.5 py-1 text-xs transition-colors ${
            props.value === opt.value
              ? "bg-[var(--accent)] text-white"
              : "text-[var(--text-secondary)] hover:bg-white/60"
          }`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
