import { useState } from "react";
import type { PresetDefinition } from "../../types/domain";

export interface PresetSelectorProps {
  presets: readonly PresetDefinition[];
  activePresetId: string | null;
  onSelect(id: string): void;
  onRename?(id: string, name: string): void;
  onDelete?(id: string): void;
}

export function PresetSelector(props: PresetSelectorProps): JSX.Element {
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");

  const startRename = (preset: PresetDefinition) => {
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
            <span key={p.id} className="flex items-center gap-1">
              <button
                onClick={() => props.onSelect(p.id)}
                className={`rounded-lg px-3 py-1.5 text-xs transition-colors ${
                  props.activePresetId === p.id
                    ? "bg-[var(--accent)] text-white"
                    : "bg-white/50 text-[var(--text-primary)] hover:bg-white/80"
                }`}
                title={p.description}
              >
                {p.name}
              </button>
              {p.custom && props.onRename && props.onDelete && (
                <span className="flex items-center gap-0.5">
                  <button
                    type="button"
                    onClick={() => startRename(p)}
                    className="rounded border border-[var(--border-hairline)] bg-white/40 px-1 py-0.5 text-[10px] text-[var(--text-secondary)] hover:bg-white/70"
                    title="重命名"
                  >
                    改名
                  </button>
                  <button
                    type="button"
                    onClick={() => props.onDelete?.(p.id)}
                    className="rounded border border-red-200 bg-red-50 px-1 py-0.5 text-[10px] text-red-600 hover:bg-red-100"
                    title="删除"
                  >
                    ×
                  </button>
                </span>
              )}
            </span>
          ),
        )}
      </div>
    </div>
  );
}