import { create } from "zustand";
import type {
  InputItem,
  RenderConfig,
  OuterShadowConfig,
  StrokeConfig,
  UpscaleConfig,
} from "../types/domain";
import { ExportMode } from "../types/domain";
import type {
  CommandError,
  ExportIcoResponse,
  ApplyToLnkResponse,
} from "../types/commands";
import { DEFAULT_RENDER_CONFIG, DEFAULT_UPSCALE_CONFIG } from "../constants/defaults";
import { getPresetById } from "../constants/presets";
import { commands } from "../lib/tauri";
import { normalizeInvokeError } from "../types/errors";

export interface IconForgeState {
  items: InputItem[];
  selectedItemId: string | null;
  renderConfig: RenderConfig;
  upscaleConfig: UpscaleConfig;
  activePresetId: string | null;
  isImporting: boolean;
  isExporting: boolean;
  dragActive: boolean;
  error: CommandError | null;
  lastExportResult: ExportIcoResponse | ApplyToLnkResponse | null;

  importPaths(paths: string[]): Promise<void>;
  removeItem(id: string): void;
  clearItems(): void;
  selectItem(id: string): void;
  updateRenderConfig(patch: Partial<RenderConfig>): void;
  updateOuterShadow(patch: Partial<OuterShadowConfig>): void;
  updateStroke(patch: Partial<StrokeConfig>): void;
  updateUpscaleConfig(patch: Partial<UpscaleConfig>): void;
  applyPreset(presetId: string): void;
  exportAsIco(): Promise<void>;
  applyToSelectedLnks(): Promise<void>;
  setDragActive(active: boolean): void;
  clearError(): void;
}

export const useIconForgeStore = create<IconForgeState>((set, get) => ({
  items: [],
  selectedItemId: null,
  renderConfig: { ...DEFAULT_RENDER_CONFIG },
  upscaleConfig: { ...DEFAULT_UPSCALE_CONFIG },
  activePresetId: "macos-classic-rounded",
  isImporting: false,
  isExporting: false,
  dragActive: false,
  error: null,
  lastExportResult: null,

  importPaths: async (paths: string[]) => {
    set({ isImporting: true, error: null });
    try {
      const response = await commands.importPaths({ paths });
      set((state) => {
        const existingPaths = new Set(
          state.items.map((i) => i.sourcePath.toLowerCase()),
        );
        const newItems = response.items.filter(
          (i) => !existingPaths.has(i.sourcePath.toLowerCase()),
        );
        const updatedItems = [...state.items, ...newItems];
        return {
          items: updatedItems,
          selectedItemId:
            state.selectedItemId ?? newItems[0]?.id ?? null,
          isImporting: false,
        };
      });
    } catch (err) {
      set({ error: normalizeInvokeError(err), isImporting: false });
    }
  },

  removeItem: (id: string) => {
    set((state) => {
      const idx = state.items.findIndex((i) => i.id === id);
      if (idx === -1) return state;
      const newItems = state.items.filter((i) => i.id !== id);
      let newSelected = state.selectedItemId;
      if (state.selectedItemId === id) {
        if (idx < newItems.length) {
          newSelected = newItems[idx]?.id ?? null;
        } else if (newItems.length > 0) {
          newSelected = newItems[newItems.length - 1]?.id ?? null;
        } else {
          newSelected = null;
        }
      }
      return { items: newItems, selectedItemId: newSelected };
    });
  },

  clearItems: () => set({ items: [], selectedItemId: null }),

  selectItem: (id: string) => set({ selectedItemId: id }),

  updateRenderConfig: (patch: Partial<RenderConfig>) => {
    set((state) => ({
      renderConfig: { ...state.renderConfig, ...patch },
      activePresetId: null,
    }));
  },

  updateOuterShadow: (patch: Partial<OuterShadowConfig>) => {
    set((state) => ({
      renderConfig: {
        ...state.renderConfig,
        outerShadow: { ...state.renderConfig.outerShadow, ...patch },
      },
      activePresetId: null,
    }));
  },

  updateStroke: (patch: Partial<StrokeConfig>) => {
    set((state) => ({
      renderConfig: {
        ...state.renderConfig,
        stroke: { ...state.renderConfig.stroke, ...patch },
      },
      activePresetId: null,
    }));
  },

  updateUpscaleConfig: (patch: Partial<UpscaleConfig>) => {
    set((state) => ({
      upscaleConfig: { ...state.upscaleConfig, ...patch },
      activePresetId: null,
    }));
  },

  applyPreset: (presetId: string) => {
    const preset = getPresetById(presetId);
    if (!preset) return;
    set({
      renderConfig: JSON.parse(JSON.stringify(preset.config)),
      upscaleConfig: { ...DEFAULT_UPSCALE_CONFIG },
      activePresetId: presetId,
    });
  },

  exportAsIco: async () => {
    const { items, renderConfig } = get();
    const selectedPaths = items
      .filter((i) => selectedPathsFilter(i, items, get().selectedItemId))
      .map((i) => i.sourcePath);
    set({ isExporting: true, error: null, lastExportResult: null });
    try {
      const result = await commands.exportIco({
        sourcePaths: selectedPaths,
        renderConfig,
        exportMode: ExportMode.ExportAsIco,
      });
      set({ lastExportResult: result, isExporting: false });
    } catch (err) {
      set({ error: normalizeInvokeError(err), isExporting: false });
    }
  },

  applyToSelectedLnks: async () => {
    const { items, renderConfig } = get();
    const lnkPaths = items
      .filter((i) => i.fileType === "Lnk")
      .map((i) => i.sourcePath);
    set({ isExporting: true, error: null, lastExportResult: null });
    try {
      const result = await commands.applyToLnk({
        lnkPaths,
        renderConfig,
        exportMode: ExportMode.ApplyToLnk,
      });
      set({ lastExportResult: result, isExporting: false });
    } catch (err) {
      set({ error: normalizeInvokeError(err), isExporting: false });
    }
  },

  setDragActive: (active: boolean) => set({ dragActive: active }),

  clearError: () => set({ error: null }),
}));

function selectedPathsFilter(
  item: InputItem,
  allItems: InputItem[],
  selectedId: string | null,
): boolean {
  if (!selectedId) return true;
  const selected = allItems.find((i) => i.id === selectedId);
  if (!selected) return true;
  return item.id === selectedId;
}
