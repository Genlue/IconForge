import { open } from "@tauri-apps/plugin-dialog";
import { useIconForgeStore } from "../../store/useIconForgeStore";

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
      <div className="flex items-center gap-2">
        <button
          onClick={async () => {
            const selected = await open({
              multiple: true,
              directory: false,
              filters: [{
                name: "支持的文件",
                extensions: ["png","jpg","jpeg","webp","bmp","gif","tif","tiff","ico","exe","dll","lnk"]
              }]
            });
            if (selected?.length) await useIconForgeStore.getState().importPaths(selected);
          }}
          className="text-xs text-[var(--accent)] hover:text-[var(--accent)]/80"
        >
          + 导入文件
        </button>
        <button
          onClick={async () => {
            const selected = await open({
              multiple: false,
              directory: true,
            });
            if (selected) await useIconForgeStore.getState().importPaths([selected]);
          }}
          className="text-xs text-[var(--accent)] hover:text-[var(--accent)]/80"
        >
          + 导入文件夹
        </button>
      </div>
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
