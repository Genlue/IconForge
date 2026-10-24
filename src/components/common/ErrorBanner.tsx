import type { CommandError } from "../../types/commands";

export interface ErrorBannerProps {
  error: CommandError | null;
  onDismiss(): void;
}

/** 浮动错误提示：瞬态表面，玻璃材质。 */
export function ErrorBanner(props: ErrorBannerProps): JSX.Element | null {
  if (!props.error) return null;
  return (
    <div className="error-toast" role="alert">
      <span className="min-w-0">{props.error.message}</span>
      <button
        type="button"
        aria-label="关闭错误提示"
        onClick={props.onDismiss}
        className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-[var(--danger-soft)] text-xs hover:opacity-80"
      >
        ×
      </button>
    </div>
  );
}
