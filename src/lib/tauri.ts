import { invoke } from "@tauri-apps/api/core";
import type {
  ImportPathsRequest,
  ImportPathsResponse,
  RenderPreviewRequest,
  RenderPreviewResponse,
  ExportIcoRequest,
  ExportIcoResponse,
  ApplyToLnkRequest,
  ApplyToLnkResponse,
} from "../types/commands";

export const commands = {
  importPaths: (request: ImportPathsRequest) =>
    invoke<ImportPathsResponse>("import_paths", { request }),
  renderPreview: (request: RenderPreviewRequest) =>
    invoke<RenderPreviewResponse>("render_preview", { request }),
  exportIco: (request: ExportIcoRequest) =>
    invoke<ExportIcoResponse>("export_ico", { request }),
  applyToLnk: (request: ApplyToLnkRequest) =>
    invoke<ApplyToLnkResponse>("apply_to_lnk", { request }),
};
