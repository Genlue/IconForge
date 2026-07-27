import type { InputItem, RenderConfig } from "../../types/domain";

export interface IconPreviewProps {
  item: InputItem;
  config: RenderConfig;
  sizePx: number;
}

export function IconPreview(props: IconPreviewProps): JSX.Element {
  return (
    <div
      className="relative"
      style={{
        width: props.sizePx,
        height: props.sizePx,
        aspectRatio: "1 / 1",
        isolation: "isolate",
      }}
    >
      <span className="text-xs text-[var(--text-secondary)]">
        {props.item.displayName}
      </span>
    </div>
  );
}
