export interface SpinnerProps {
  label?: string;
}

export function Spinner(props: SpinnerProps): JSX.Element {
  return (
    <div className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
      <svg
        className="animate-spin"
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
      >
        <circle
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          strokeWidth="4"
          opacity="0.25"
        />
        <path
          d="M12 2a10 10 0 0 1 10 10"
          stroke="currentColor"
          strokeWidth="4"
          strokeLinecap="round"
        />
      </svg>
      {props.label && <span>{props.label}</span>}
    </div>
  );
}
