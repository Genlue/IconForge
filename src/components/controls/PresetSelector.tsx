import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
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
 * 预设以检查器选择列表呈现：选中行使用唯一的强调色填充，
 * 自定义预设带「自定」徽标；右键自定义预设弹出重命名 / 删除菜单。
 */
export function PresetSelector(props: PresetSelectorProps): JSX.Element {
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [menu, setMenu] = useState<MenuState | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menu) return;
    const onPointerDown = (e: PointerEvent) => {
      if (e.button !== 0) return;
      if (e.defaultPrevented) return;
      if (menuRef.current && menuRef.current.contains(e.target as Node)) return;
      setMenu(null);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setMenu(null);
    };
    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("keydown", onKey);
    };
  }, [menu]);

  const openMenu = (event: React.MouseEvent, preset: PresetDefinition) => {
    event.preventDefault();
    // 贴边时向内收，避免菜单超出窗口被裁切
    const MENU_W = 148;
    const MENU_H = 76;
    setMenu({
      preset,
      x: Math.min(event.clientX, window.innerWidth - MENU_W - 8),
      y: Math.min(event.clientY, window.innerHeight - MENU_H - 8),
    });
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
    <div className="flex flex-col gap-2">
      <div className="preset-list">
        {props.presets.map((p) =>
          renamingId === p.id ? (
            <div key={p.id} className="preset-rename">
              <input
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") commitRename(p);
                  if (e.key === "Escape") setRenamingId(null);
                }}
                autoFocus
              />
              <button
                type="button"
                onClick={() => commitRename(p)}
                className="btn btn--sm"
              >
                确定
              </button>
            </div>
          ) : (
            <button
              key={p.id}
              type="button"
              onClick={() => props.onSelect(p.id)}
              onContextMenu={(e) => {
                if (p.custom && props.onRename && props.onDelete) openMenu(e, p);
              }}
              title={p.custom ? `${p.description}（右键管理）` : p.description}
              className={`preset-row${props.activePresetId === p.id ? " preset-row--active" : ""}`}
            >
              <span className="preset-row__name">{p.name}</span>
              {p.custom && <span className="preset-row__badge">自定</span>}
            </button>
          ),
        )}
      </div>

      {menu &&
        createPortal(
          <div
            ref={menuRef}
            role="menu"
            className="context-menu fixed z-[100]"
            style={{ left: menu.x, top: menu.y }}
          >
            <button
              type="button"
              role="menuitem"
              onClick={() => startRename(menu.preset)}
              className="context-menu__item"
            >
              重命名…
            </button>
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                props.onDelete?.(menu.preset.id);
                setMenu(null);
              }}
              className="context-menu__item context-menu__item--danger"
            >
              删除
            </button>
          </div>,
          document.body,
        )}
    </div>
  );
}
