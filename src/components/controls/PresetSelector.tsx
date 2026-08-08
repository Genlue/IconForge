import { useEffect, useRef, useState } from "react";
import type { PresetDefinition } from "../../types/domain";

export interface PresetSelectorProps {
  presets: readonly PresetDefinition[];
  activePresetId: string | null;
  onSelect(id: string): void;
  onRename?(id: string, name: string): void;
  onDelete?(id: string): void;
}

interface MenuState {
  preset: PresetDefinition;
  x: number;
  y: number;
}

/**
 * Built-in presets render in the neutral chip style; user-created presets use
 * a violet accent so they are easy to tell apart. Right-clicking a custom
 * preset opens a context menu with rename / delete.
 */
export function PresetSelector(props: PresetSelectorProps): JSX.Element {
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [menu, setMenu] = useState<MenuState | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setMenu(null);
    };
    window.addEventListener("pointerdown", close, true);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", close, true);
      window.removeEventListener("keydown", onKey);
    };
  }, [menu]);

  const openMenu = (event: React.MouseEvent, preset: PresetDefinition) => {
    event.preventDefault();
    event.stopPropagation();
    setMenu({ preset, x: event.clientX, y: event.clientY });
  };

  const startRename = (preset: PresetDefinition) => {
    setMenu(null);
    setRenamingId(preset.id);
    setDraft(preset.name);
  };

  const commitRename = (preset: PresetDefinition) => {
    props.onRename?.(preset.id, draft);
    setRenamingId(null);
  };

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-2">
        {props.presets.map((p) =>
          renamingId === p.id ? (
            <div
              key={p.id}
              className="flex items-center gap-1 rounded-lg border border-[var(--accent)] bg-white/60 px-1 py-0.5"
            >
              <input
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") commitRename(p);
                  if (e.key === "Escape") setRenamingId(null);
                }}
                className="w-28 rounded border border-[var(--border-hairline)] bg-white px-1.5 py-0.5 text-xs text-[var(--text-primary)]"
                autoFocus
              />
              <button
                type="button"
                onClick={() => commitRename(p)}
                className="rounded bg-[var(--accent)] px-1.5 py-0.5 text-[10px] text-white"
              >
                确定
              </button>
            </div>
          ) : (
            <button
              key={p.id}
              onClick={() => props.onSelect(p.id)}
              onContextMenu={(e) => {
                if (p.custom && props.onRename && props.onDelete) openMenu(e, p);
              }}
              title={p.custom ? `${p.description}（右键管理）` : p.description}
              className={`max-w-[180px] truncate rounded-lg px-3 py-1.5 text-xs transition-colors ${
                p.custom
                  ? props.activePresetId === p.id
                    ? "bg-violet-500 text-white"
                    : "border border-violet-300 bg-violet-50 text-violet-700 hover:bg-violet-100"
                  : props.activePresetId === p.id
                    ? "bg-[var(--accent)] text-white"
                    : "bg-white/50 text-[var(--text-primary)] hover:bg-white/80"
              }`}
            >
              {p.name}
            </button>
          ),
        )}
      </div>

      {menu && (
        <div
          ref={menuRef}
          className="fixed z-50 min-w-[9rem] overflow-hidden rounded-lg border border-[var(--border-hairline)] bg-white shadow-xl"
          style={{ left: menu.x, top: menu.y }}
          onPointerDown={(e) => e.stopPropagation()}
        >
          <button
            type="button"
            onClick={() => startRename(menu.preset)}
            className="block w-full px-3 py-2 text-left text-xs text-[var(--text-primary)] hover:bg-violet-50"
          >
            重命名…
          </button>
          <button
            type="button"
            onClick={() => {
              props.onDelete?.(menu.preset.id);
              setMenu(null);
            }}
            className="block w-full px-3 py-2 text-left text-xs text-red-600 hover:bg-red-50"
          >
            删除
          </button>
        </div>
      )}
    </div>
  );
}