export interface EmptyDropZoneProps {
  dragActive: boolean;
}

/** 空态：内容平面上的虚线拖放目标 —— 边界即语义（拖放区），无玻璃无卡片。 */
export function EmptyDropZone(props: EmptyDropZoneProps): JSX.Element {
  return (
    <div className={`drop-target${props.dragActive ? " drop-target--active" : ""}`}>
      <svg
        width="40"
        height="40"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.4"
        aria-hidden="true"
      >
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="17 8 12 3 7 8" />
        <line x1="12" y1="3" x2="12" y2="15" />
      </svg>
      <p className="text-[13px] font-medium">拖入图片、EXE、DLL、LNK</p>
      <p className="text-[11px] opacity-80">或点工具栏「导入」(Ctrl+O)</p>
    </div>
  );
}
