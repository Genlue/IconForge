import type { BrushStroke, InputItem, RenderConfig, UpscaleConfig, WandStroke, ExportMode } from "./domain";

export interface ImportPathsRequest {
  paths: string[];
}

export interface ImportPathsResponse {
  items: InputItem[];
  rejectedPaths: RejectedPath[];
  warnings: string[];
}

export interface RejectedPath {
  path: string;
  code: string;
  message: string;
}

export interface RenderPreviewRequest {
  sourcePath: string;
  renderConfig: RenderConfig;
  upscaleConfig: UpscaleConfig;
  brushStrokes: BrushStroke[];
  wandStrokes: WandStroke[];
  previewSize: number;
}

export interface RenderPreviewResponse {
  pngBase64: string;
  width: number;
  height: number;
  processedSourceWidth: number;
  processedSourceHeight: number;
}

export interface ExportIcoItemRequest {
  sourcePath: string;
  renderConfig: RenderConfig;
  upscaleConfig: UpscaleConfig;
  brushStrokes: BrushStroke[];
  wandStrokes: WandStroke[];
}

export interface ExportIcoRequest {
  items: ExportIcoItemRequest[];
  exportMode: ExportMode.ExportAsIco;
}

export interface ExportPngRequest {
  items: ExportIcoItemRequest[];
  rawSource: boolean;
}

export interface ExportedFile {
  sourcePath: string;
  outputPath: string;
}

export interface ExportIcoResponse {
  cancelled: boolean;
  files: ExportedFile[];
  warnings: string[];
}

export interface ApplyToLnkItemRequest {
  lnkPath: string;
  renderConfig: RenderConfig;
  upscaleConfig: UpscaleConfig;
  brushStrokes: BrushStroke[];
  wandStrokes: WandStroke[];
}

export interface ApplyToLnkRequest {
  items: ApplyToLnkItemRequest[];
  exportMode: ExportMode.ApplyToLnk;
}

export interface AppliedShortcut {
  lnkPath: string;
  managedIconPath: string;
  backupPath: string | null;
}

export interface ApplyToLnkResponse {
  applied: AppliedShortcut[];
  failed: RejectedPath[];
}

export interface CommandError {
  code:
    | "InvalidArgument"
    | "PathNotFound"
    | "UnsupportedImage"
    | "IconExtractionFailed"
    | "ShortcutReadFailed"
    | "ShortcutWriteFailed"
    | "RenderFailed"
    | "IcoEncodeFailed"
    | "IoFailed"
    | "PlatformUnsupported"
    | "Internal";
  message: string;
  path: string | null;
  details: string | null;
}
