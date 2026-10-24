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
      role="button"
      tabIndex={0}
      onClick={() => props.onSelect(props.item.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          props.onSelect(props.item.id);
        }
      }}
      title={props.item.displayName}
      className={`source-row${props.selected ? " source-row--selected" : ""}`}
    >
      <img
        src={thumbSrc}
        alt=""
        className="h-6 w-6 shrink-0 rounded-[5px] object-contain"
      />
      <span className="source-row__name">{props.item.displayName}</span>
      <span className="source-row__meta">
        {props.item.sourceWidth}×{props.item.sourceHeight}
      </span>
      <button
        type="button"
        aria-label={`移除 ${props.item.displayName}`}
        title={`移除 ${props.item.displayName}`}
        onClick={(e) => {
          e.stopPropagation();
          props.onRemove(props.item.id);
        }}
        className="source-row__remove"
      >
        ×
      </button>
    </div>
  );
}

export const BatchItem = memo(BatchItemInner);
