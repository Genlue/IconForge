import { useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const appWindow = getCurrentWebviewWindow();

const trafficBtn =
  "flex items-center justify-center h-3 w-3 rounded-full border-none outline-none p-0 cursor-pointer";

export function TitleBar(): JSX.Element {
  const [hoverRed, setHoverRed] = useState(false);
  const [hoverYellow, setHoverYellow] = useState(false);
  const [hoverGreen, setHoverGreen] = useState(false);

  return (
    <header
      data-tauri-drag-region="true"
      className="flex h-[52px] items-center gap-3 px-4"
      style={{ WebkitUserSelect: "none", userSelect: "none" }}
      onDoubleClick={() => appWindow.toggleMaximize()}
    >
      {/* macOS-style traffic light buttons */}
      <div className="flex items-center gap-[6px]" data-tauri-drag-region="false">
        <button
          className={`${trafficBtn} bg-[#EC6A5E] text-[10px] leading-none text-[#4d1a1a]`}
          onClick={() => appWindow.close()}
          onMouseEnter={() => setHoverRed(true)}
          onMouseLeave={() => setHoverRed(false)}
          aria-label="关闭"
        >
          {hoverRed && <span>&#10005;</span>}
        </button>
        <button
          className={`${trafficBtn} bg-[#F5BF4F] text-[10px] leading-none text-[#594a1a]`}
          onClick={() => appWindow.minimize()}
          onMouseEnter={() => setHoverYellow(true)}
          onMouseLeave={() => setHoverYellow(false)}
          aria-label="最小化"
        >
          {hoverYellow && <span>&#8722;</span>}
        </button>
        <button
          className={`${trafficBtn} bg-[#62C554] text-[10px] leading-none text-[#1a4d1a]`}
          onClick={() => appWindow.toggleMaximize()}
          onMouseEnter={() => setHoverGreen(true)}
          onMouseLeave={() => setHoverGreen(false)}
          aria-label="最大化"
        >
          {hoverGreen && <span>&#9633;</span>}
        </button>
      </div>
      <span className="ml-2 text-[13px] font-semibold text-[var(--text-primary)]">
        IconForge
      </span>
    </header>
  );
}
