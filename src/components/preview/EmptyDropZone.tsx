export interface EmptyDropZoneProps {
  dragActive: boolean;
}

export function EmptyDropZone(props: EmptyDropZoneProps): JSX.Element {
  return (
    <div
      className={`empty-preview-state flex flex-col items-center justify-center gap-3 transition-opacity ${
        props.dragActive ? "opacity-60" : "opacity-100"
      }`}
    >
      <svg
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        className="text-[var(--empty-state-text)]"
      >
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="17 8 12 3 7 8" />
        <line x1="12" y1="3" x2="12" y2="15" />
      </svg>
      <p className="text-sm font-medium text-[var(--empty-state-text)]">
        拖入图片、EXE、DLL、LNK
      </p>
    </div>
  );
}
