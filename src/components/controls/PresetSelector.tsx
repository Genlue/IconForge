import type { PresetDefinition } from "../../types/domain";

export interface PresetSelectorProps {
  presets: readonly PresetDefinition[];
  activePresetId: string | null;
  onSelect(id: string): void;
}

export function PresetSelector(props: PresetSelectorProps): JSX.Element {
  return (
    <div className="flex flex-wrap gap-2">
      {props.presets.map((p) => (
        <button
          key={p.id}
          onClick={() => props.onSelect(p.id)}
          className={`rounded-lg px-3 py-1.5 text-xs transition-colors ${
            props.activePresetId === p.id
              ? "bg-[var(--accent)] text-white"
              : "bg-white/50 text-[var(--text-primary)] hover:bg-white/80"
          }`}
        >
          {p.name}
        </button>
      ))}
    </div>
  );
}
