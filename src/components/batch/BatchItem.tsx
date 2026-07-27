import { memo, useMemo } from "react";
import type { InputItem } from "../../types/domain";

export interface BatchItemProps {
  item: InputItem;
  selected: boolean;
  onSelect(id: string): void;
  onRemove(id: string): void;
}

function BatchItemInner(props: BatchItemProps): JSX.Element {
  const thumbSrc = useMemo(
    () => `data:image/png;base64,${props.item.thumbnailPngBase64}`,
    [props.item.thumbnailPngBase64],
  );

  return (
    <div
      onClick={() => props.onSelect(props.item.id)}
      className={`relative flex h-[132px] w-[132px] shrink-0 cursor-pointer flex-col items-center justify-center gap-1 rounded-xl border transition-colors ${
        props.selected
          ? "border-[var(--accent)] bg-[var(--accent)]/10"
          : "border-[var(--border-hairline)] bg-white/40 hover:bg-white/60"
      }`}
    >
      <img
        src={thumbSrc}
        alt={props.item.displayName}
        className="h-[72px] w-[72px] rounded object-contain"
      />
      <span className="max-w-[120px] truncate text-[10px] text-[var(--text-secondary)]">
        {props.item.displayName}
      </span>
      <button
        onClick={(e) => {
          e.stopPropagation();
          props.onRemove(props.item.id);
        }}
        className="absolute right-1 top-1 flex h-4 w-4 items-center justify-center rounded-full bg-black/10 text-[10px] text-[var(--text-secondary)] hover:bg-black/20"
      >
        ×
      </button>
    </div>
  );
}

export const BatchItem = memo(BatchItemInner);
