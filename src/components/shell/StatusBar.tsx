export interface StatusBarProps {
  itemCount: number;
  busy: boolean;
  message: string | null;
}

export function StatusBar(props: StatusBarProps): JSX.Element {
  return (
    <div className="flex h-6 items-center justify-between px-3 text-xs text-[var(--text-secondary)]">
      <span>{props.busy ? "处理中..." : props.message ?? `${props.itemCount} 项`}</span>
    </div>
  );
}
