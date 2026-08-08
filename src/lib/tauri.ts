import { invoke } from "@tauri-apps/api/core";
import type {
  ImportPathsRequest,
  ImportPathsResponse,
  RenderPreviewRequest,
  RenderPreviewResponse,
  ExportIcoRequest,
  ExportIcoResponse,
  ExportPngRequest,
  ApplyToLnkRequest,
  ApplyToLnkResponse,
  WandSelectionRequest,
  WandSelectionResponse,
  SourceImageRequest,
  SourceImageResponse,
} from "../types/commands";

export const commands = {
  importPaths: (request: ImportPathsRequest) =>
    invoke<ImportPathsResponse>("import_paths", { request }),
  renderPreview: (request: RenderPreviewRequest) =>
    invoke<RenderPreviewResponse>("render_preview", { request }),
  computeWandSelection: (request: WandSelectionRequest) =>
    invoke<WandSelectionResponse>("compute_wand_selection", { request }),
  renderSourceImage: (request: SourceImageRequest) =>
    invoke<SourceImageResponse>("render_source_image", { request }),
  exportIco: (request: ExportIcoRequest) =>
    invoke<ExportIcoResponse>("export_ico", { request }),
  exportPng: (request: ExportPngRequest) =>
    invoke<ExportIcoResponse>("export_png", { request }),
  applyToLnk: (request: ApplyToLnkRequest) =>
    invoke<ApplyToLnkResponse>("apply_to_lnk", { request }),
};
