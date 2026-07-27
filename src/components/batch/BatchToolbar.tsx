export interface BatchToolbarProps {
  itemCount: number;
  onClear(): void;
}

export function BatchToolbar(props: BatchToolbarProps): JSX.Element {
  return (
    <div className="flex items-center justify-between px-2">
      <span className="text-xs text-[var(--text-secondary)]">
        {props.itemCount} 项
      </span>
      {props.itemCount > 0 && (
        <button
          onClick={props.onClear}
          className="text-xs text-red-500 hover:text-red-600"
        >
          清空
        </button>
      )}
    </div>
  );
}
