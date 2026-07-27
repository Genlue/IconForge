import type { InputItem, RenderConfig, ExportMode } from "./domain";

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
  previewSize: number;
}

export interface RenderPreviewResponse {
  pngBase64: string;
  width: number;
  height: number;
}

export interface ExportIcoRequest {
  sourcePaths: string[];
  renderConfig: RenderConfig;
  exportMode: ExportMode.ExportAsIco;
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

export interface ApplyToLnkRequest {
  lnkPaths: string[];
  renderConfig: RenderConfig;
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
