import type { CommandError } from "../../types/commands";

export interface ErrorBannerProps {
  error: CommandError | null;
  onDismiss(): void;
}

export function ErrorBanner(props: ErrorBannerProps): JSX.Element | null {
  if (!props.error) return null;
  return (
    <div className="fixed bottom-4 left-1/2 z-50 flex -translate-x-1/2 items-center gap-3 rounded-xl bg-red-500/90 px-4 py-2 text-sm text-white shadow-lg backdrop-blur-sm">
      <span>{props.error.message}</span>
      <button
        onClick={props.onDismiss}
        className="ml-2 flex h-5 w-5 items-center justify-center rounded-full bg-white/20 text-xs hover:bg-white/30"
      >
        ×
      </button>
    </div>
  );
}
