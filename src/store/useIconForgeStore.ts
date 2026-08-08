import { create } from "zustand";
import type {
  InputItem,
  ItemConfig,
  BrushStroke,
  ColorPickTarget,
  PreviewTool,
  RenderConfig,
  OuterShadowConfig,
  StrokeConfig,
  UpscaleConfig,
  HqRenderConfig,
  WandStroke,
  WandSelection,
  EraserStroke,
  PresetDefinition,
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

const CUSTOM_PRESETS_KEY = "iconforge.customPresets.v1";

function loadCustomPresets(): PresetDefinition[] {
  try {
    if (typeof window === "undefined") return [];
    const raw = window.localStorage.getItem(CUSTOM_PRESETS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function persistCustomPresets(presets: PresetDefinition[]): void {
  try {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(CUSTOM_PRESETS_KEY, JSON.stringify(presets));
  } catch {
    // storage unavailable: presets stay in memory for the session
  }
}

export interface IconForgeState {
  items: InputItem[];
  selectedItemId: string | null;
  itemConfigs: Record<string, ItemConfig>;
  previewTool: PreviewTool;
  colorPickTarget: ColorPickTarget | null;
  wandSelection: WandSelection | null;
  sourceEditorOpen: boolean;
  isImporting: boolean;
  isExporting: boolean;
  dragActive: boolean;
  error: CommandError | null;
  lastExportResult: ExportIcoResponse | ApplyToLnkResponse | null;
  customPresets: PresetDefinition[];

  importPaths(paths: string[]): Promise<void>;
  removeItem(id: string): void;
  clearItems(): void;
  selectItem(id: string): void;
  updateRenderConfig(patch: Partial<RenderConfig>): void;
  updateOuterShadow(patch: Partial<OuterShadowConfig>): void;
  updateStroke(patch: Partial<StrokeConfig>): void;
  updateHqRender(patch: Partial<HqRenderConfig>): void;
  updateUpscaleConfig(patch: Partial<UpscaleConfig>): void;
  applyPreset(presetId: string): void;
  applyCurrentConfigToAll(): void;
  setPreviewTool(tool: PreviewTool): void;
  startColorPicking(target: ColorPickTarget): void;
  applyPickedColor(rgbHex: string): void;
  addBrushStroke(stroke: BrushStroke): void;
  undoBrushStroke(): void;
  clearBrushStrokes(): void;
  updateBrushSettings(patch: { color?: string; size?: number; mode?: "paint" | "erase"; clipToMask?: boolean }): void;
  setWandTolerance(tolerance: number): void;
  setWandSelection(selection: WandSelection | null): void;
  clearWandSelection(): void;
  deleteWandSelection(): void;
  addWandStroke(stroke: WandStroke): void;
  undoWandStroke(): void;
  clearWandStrokes(): void;
  updateEraserSettings(patch: { size?: number; hardness?: number }): void;
  addEraserStroke(stroke: EraserStroke): void;
  undoEraserStroke(): void;
  clearEraserStrokes(): void;
  exportAsIco(): Promise<void>;
  exportAllAsIco(): Promise<void>;
  exportAsPng(): Promise<void>;
  exportAllAsPng(): Promise<void>;
  extractIcoAsPng(): Promise<void>;
  applyToSelectedLnks(): Promise<void>;
  setDragActive(active: boolean): void;
  setSourceEditorOpen(open: boolean): void;
  clearError(): void;
  saveCurrentAsPreset(): string | null;
  renamePreset(id: string, name: string): void;
  deletePreset(id: string): void;
}

export const useIconForgeStore = create<IconForgeState>((set, get) => ({
  items: [],
  selectedItemId: null,
  itemConfigs: {},
  previewTool: "none",
  colorPickTarget: null,
  wandSelection: null,
  sourceEditorOpen: false,
  isImporting: false,
  isExporting: false,
  dragActive: false,
  error: null,
  lastExportResult: null,
  customPresets: loadCustomPresets(),

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
        const sourceConfig = state.selectedItemId
          ? state.itemConfigs[state.selectedItemId]
          : undefined;
        const itemConfigs = { ...state.itemConfigs };
        for (const item of newItems) {
          itemConfigs[item.id] = cloneItemConfig(sourceConfig ?? createDefaultItemConfig());
        }
        return {
          items: updatedItems,
          itemConfigs,
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
      const itemConfigs = { ...state.itemConfigs };
      delete itemConfigs[id];
      return { items: newItems, selectedItemId: newSelected, itemConfigs };
    });
  },

  clearItems: () => set({ items: [], selectedItemId: null, itemConfigs: {} }),

  selectItem: (id: string) => set({ selectedItemId: id }),

  updateRenderConfig: (patch: Partial<RenderConfig>) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      renderConfig: { ...config.renderConfig, ...patch },
      activePresetId: null,
    })));
  },

  updateOuterShadow: (patch: Partial<OuterShadowConfig>) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      renderConfig: {
        ...config.renderConfig,
        outerShadow: { ...config.renderConfig.outerShadow, ...patch },
      },
      activePresetId: null,
    })));
  },

  updateStroke: (patch: Partial<StrokeConfig>) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      renderConfig: {
        ...config.renderConfig,
        stroke: { ...config.renderConfig.stroke, ...patch },
      },
      activePresetId: null,
    })));
  },

  updateHqRender: (patch: Partial<HqRenderConfig>) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      renderConfig: {
        ...config.renderConfig,
        hqRender: { ...config.renderConfig.hqRender, ...patch },
      },
      activePresetId: null,
    })));
  },

  updateUpscaleConfig: (patch: Partial<UpscaleConfig>) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      upscaleConfig: { ...config.upscaleConfig, ...patch },
    })));
  },

  applyPreset: (presetId: string) => {
    const preset =
      getPresetById(presetId) ?? get().customPresets.find((p) => p.id === presetId);
    if (!preset) return;
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      renderConfig: cloneRenderConfig(preset.config),
      activePresetId: presetId,
    })));
  },

  applyCurrentConfigToAll: () => {
    set((state) => {
      if (!state.selectedItemId || state.items.length < 2) return state;
      const selectedConfig = state.itemConfigs[state.selectedItemId];
      if (!selectedConfig) return state;
      const itemConfigs: Record<string, ItemConfig> = {};
      for (const item of state.items) {
        itemConfigs[item.id] = cloneItemConfig(selectedConfig);
      }
      return { itemConfigs };
    });
  },

  setPreviewTool: (previewTool) => set({ previewTool, colorPickTarget: null }),

  startColorPicking: (colorPickTarget) => set({
    previewTool: "eyedropper",
    colorPickTarget,
  }),

  applyPickedColor: (rgbHex) => {
    set((state) => updateSelectedConfig(state, (config) => {
      const target = state.colorPickTarget;
      if (!target) return config;
      const withAlpha = (current: string) => `${rgbHex}${current.slice(7, 9) || "FF"}`;
      if (target === "brush") {
        return { ...config, brushColor: withAlpha(config.brushColor) };
      }
      if (target === "shadow") {
        return {
          ...config,
          renderConfig: {
            ...config.renderConfig,
            outerShadow: {
              ...config.renderConfig.outerShadow,
              color: withAlpha(config.renderConfig.outerShadow.color),
            },
          },
          activePresetId: null,
        };
      }
      if (target === "stroke") {
        return {
          ...config,
          renderConfig: {
            ...config.renderConfig,
            stroke: {
              ...config.renderConfig.stroke,
              color: withAlpha(config.renderConfig.stroke.color),
            },
          },
          activePresetId: null,
        };
      }
      if (target === "glossLight" || target === "glossDark") {
        const key = target === "glossLight" ? "lightColor" : "darkColor";
        return {
          ...config,
          renderConfig: {
            ...config.renderConfig,
            gloss: {
              ...config.renderConfig.gloss,
              [key]: withAlpha(config.renderConfig.gloss[key]),
            },
          },
          activePresetId: null,
        };
      }
      const key = target === "backplate"
        ? "backplateColor"
        : target === "gradientStart"
          ? "gradientStartColor"
          : "gradientEndColor";
      return {
        ...config,
        renderConfig: {
          ...config.renderConfig,
          [key]: withAlpha(config.renderConfig[key]),
        },
        activePresetId: null,
      };
    }));
    set({ previewTool: "none", colorPickTarget: null });
  },

  addBrushStroke: (stroke) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      brushStrokes: [...config.brushStrokes, cloneBrushStroke(stroke)],
      activePresetId: null,
    })));
  },

  undoBrushStroke: () => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      brushStrokes: config.brushStrokes.slice(0, -1),
    })));
  },

  clearBrushStrokes: () => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      brushStrokes: [],
    })));
  },

  updateBrushSettings: (patch) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      brushColor: patch.color ?? config.brushColor,
      brushSize: patch.size ?? config.brushSize,
      brushMode: patch.mode ?? config.brushMode,
      brushClipToMask: patch.clipToMask ?? config.brushClipToMask,
    })));
  },

  setWandTolerance: (wandTolerance) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      wandTolerance,
    })));
  },

  addWandStroke: (stroke) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      wandStrokes: [...config.wandStrokes, cloneWandStroke(stroke)],
    })));
  },

  undoWandStroke: () => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      wandStrokes: config.wandStrokes.slice(0, -1),
    })));
  },

  clearWandStrokes: () => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      wandStrokes: [],
    })));
  },

  setWandSelection: (selection: WandSelection | null) => set({ wandSelection: selection }),

  clearWandSelection: () => set({ wandSelection: null }),

  deleteWandSelection: () => {
    const { wandSelection, previewTool } = get();
    if (!wandSelection || previewTool !== "magic-wand") return;
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      wandStrokes: [
        ...config.wandStrokes,
        {
          points: [{ x: wandSelection.x, y: wandSelection.y }],
          tolerance: wandSelection.tolerance,
        },
      ],
    })));
    set({ wandSelection: null });
  },

  updateEraserSettings: (patch) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      eraserSize: patch.size ?? config.eraserSize,
      eraserHardness: patch.hardness ?? config.eraserHardness,
    })));
  },

  addEraserStroke: (stroke) => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      eraserStrokes: [...config.eraserStrokes, cloneEraserStroke(stroke)],
    })));
  },

  undoEraserStroke: () => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      eraserStrokes: config.eraserStrokes.slice(0, -1),
    })));
  },

  clearEraserStrokes: () => {
    set((state) => updateSelectedConfig(state, (config) => ({
      ...config,
      eraserStrokes: [],
    })));
  },

  exportAsIco: async () => {
    const { items, itemConfigs, selectedItemId } = get();
    const selectedItems = items
      .filter((i) => selectedPathsFilter(i, items, selectedItemId));
    await exportItemsAsIco(selectedItems, itemConfigs, set);
  },

  exportAllAsIco: async () => {
    const { items, itemConfigs } = get();
    await exportItemsAsIco(items, itemConfigs, set);
  },

  exportAsPng: async () => {
    const { items, itemConfigs, selectedItemId } = get();
    await exportItemsAsPng(items.filter((item) => selectedPathsFilter(item, items, selectedItemId)), itemConfigs, set, false);
  },

  exportAllAsPng: async () => {
    const { items, itemConfigs } = get();
    await exportItemsAsPng(items, itemConfigs, set, false);
  },

  extractIcoAsPng: async () => {
    const { items, itemConfigs } = get();
    const icoItems = items.filter((item) => item.sourcePath.toLowerCase().endsWith(".ico"));
    await exportItemsAsPng(icoItems, itemConfigs, set, true);
  },

  applyToSelectedLnks: async () => {
    const { items, itemConfigs } = get();
    const lnkItems = items
      .filter((i) => i.fileType === "Lnk")
      .flatMap((item) => {
        const config = itemConfigs[item.id];
        return config ? [{
          lnkPath: item.sourcePath,
          renderConfig: config.renderConfig,
          upscaleConfig: config.upscaleConfig,
          brushStrokes: config.brushStrokes,
          wandStrokes: config.wandStrokes,
          eraserStrokes: config.eraserStrokes,
        }] : [];
      });
    set({ isExporting: true, error: null, lastExportResult: null });
    try {
      const result = await commands.applyToLnk({
        items: lnkItems,
        exportMode: ExportMode.ApplyToLnk,
      });
      set({ lastExportResult: result, isExporting: false });
    } catch (err) {
      set({ error: normalizeInvokeError(err), isExporting: false });
    }
  },

  setDragActive: (active: boolean) => set({ dragActive: active }),

  setSourceEditorOpen: (sourceEditorOpen) => set({ sourceEditorOpen }),

  clearError: () => set({ error: null }),

  saveCurrentAsPreset: () => {
    const { selectedItemId, itemConfigs, customPresets } = get();
    const config = selectedItemId ? itemConfigs[selectedItemId] : undefined;
    if (!config) return null;
    const presetId = `custom-${Date.now().toString(36)}`;
    const base = cloneRenderConfig(config.renderConfig);
    // The source-processing category (auto cutout) is deliberately not part
    // of a saved preset; upscale and brush settings are per-icon anyway.
    base.autoCutout = {
      enabled: false,
      tolerance: 20,
      feather: 8,
    };
    const nextNumber = customPresets.reduce((max, p) => {
      const m = /^自定义预设 (\d+)$/.exec(p.name);
      return m ? Math.max(max, Number(m[1])) : max;
    }, 0) + 1;
    const preset: PresetDefinition = {
      id: presetId,
      name: `自定义预设 ${nextNumber}`,
      description: "保存的自定义设置",
      config: base,
      custom: true,
    };
    const next = [...customPresets, preset];
    persistCustomPresets(next);
    set((state) => {
      if (!state.selectedItemId) return { customPresets: next };
      const current = state.itemConfigs[state.selectedItemId];
      if (!current) return { customPresets: next };
      return {
        customPresets: next,
        itemConfigs: {
          ...state.itemConfigs,
          [state.selectedItemId]: { ...current, activePresetId: presetId },
        },
      };
    });
    return presetId;
  },

  renamePreset: (id, name) => {
    const trimmed = name.trim();
    if (!trimmed) return;
    set((state) => {
      const next = state.customPresets.map((p) =>
        p.id === id ? { ...p, name: trimmed } : p,
      );
      persistCustomPresets(next);
      return { customPresets: next };
    });
  },

  deletePreset: (id) => {
    set((state) => {
      const next = state.customPresets.filter((p) => p.id !== id);
      persistCustomPresets(next);
      const patch: Partial<IconForgeState> = { customPresets: next };
      // Clear the active preset marker when the deleted preset was in use.
      if (state.selectedItemId && state.itemConfigs[state.selectedItemId]?.activePresetId === id) {
        const current = state.itemConfigs[state.selectedItemId];
        if (current) {
          patch.itemConfigs = {
            ...state.itemConfigs,
            [state.selectedItemId]: { ...current, activePresetId: null },
          };
        }
      }
      return patch;
    });
  },
}));

async function exportItemsAsIco(
  items: InputItem[],
  itemConfigs: Record<string, ItemConfig>,
  set: (patch: Partial<IconForgeState>) => void,
): Promise<void> {
  const exportItems = items.flatMap((item) => {
    const config = itemConfigs[item.id];
    return config ? [{
      sourcePath: item.sourcePath,
      renderConfig: config.renderConfig,
      upscaleConfig: config.upscaleConfig,
      brushStrokes: config.brushStrokes,
      wandStrokes: config.wandStrokes,
      eraserStrokes: config.eraserStrokes,
    }] : [];
  });
  set({ isExporting: true, error: null, lastExportResult: null });
  try {
    const result = await commands.exportIco({
      items: exportItems,
      exportMode: ExportMode.ExportAsIco,
    });
    set({
      lastExportResult: result.cancelled ? null : result,
      isExporting: false,
    });
  } catch (err) {
    set({ error: normalizeInvokeError(err), isExporting: false });
  }
}

function createDefaultItemConfig(): ItemConfig {
  return {
    renderConfig: cloneRenderConfig(DEFAULT_RENDER_CONFIG),
    upscaleConfig: { ...DEFAULT_UPSCALE_CONFIG },
    brushStrokes: [],
    brushColor: "#FF3B30FF",
    brushSize: 8,
    brushMode: "paint",
    brushClipToMask: true,
    wandStrokes: [],
    wandTolerance: 72,
    eraserStrokes: [],
    eraserSize: 16,
    eraserHardness: 50,
    activePresetId: "macos-classic-rounded",
  };
}

function cloneRenderConfig(config: RenderConfig): RenderConfig {
  return {
    ...config,
    outerShadow: { ...config.outerShadow },
    stroke: { ...config.stroke },
    gloss: { ...config.gloss },
    autoCutout: { ...config.autoCutout },
    hqRender: { ...config.hqRender },
  };
}

function cloneItemConfig(config: ItemConfig): ItemConfig {
  return {
    renderConfig: cloneRenderConfig(config.renderConfig),
    upscaleConfig: { ...config.upscaleConfig },
    brushStrokes: config.brushStrokes.map(cloneBrushStroke),
    brushColor: config.brushColor,
    brushSize: config.brushSize,
    brushMode: config.brushMode,
    brushClipToMask: config.brushClipToMask,
    wandStrokes: config.wandStrokes.map(cloneWandStroke),
    wandTolerance: config.wandTolerance,
    eraserStrokes: config.eraserStrokes.map(cloneEraserStroke),
    eraserSize: config.eraserSize,
    eraserHardness: config.eraserHardness,
    activePresetId: config.activePresetId,
  };
}

async function exportItemsAsPng(
  items: InputItem[],
  itemConfigs: Record<string, ItemConfig>,
  set: (patch: Partial<IconForgeState>) => void,
  rawSource: boolean,
): Promise<void> {
  const exportItems = items.flatMap((item) => {
    const config = itemConfigs[item.id];
    return config ? [{ sourcePath: item.sourcePath, renderConfig: config.renderConfig, upscaleConfig: config.upscaleConfig, brushStrokes: config.brushStrokes, wandStrokes: config.wandStrokes, eraserStrokes: config.eraserStrokes }] : [];
  });
  if (exportItems.length === 0) return;
  set({ isExporting: true, error: null, lastExportResult: null });
  try {
    const result = await commands.exportPng({ items: exportItems, rawSource });
    set({ lastExportResult: result.cancelled ? null : result, isExporting: false });
  } catch (err) {
    set({ error: normalizeInvokeError(err), isExporting: false });
  }
}

function cloneBrushStroke(stroke: BrushStroke): BrushStroke {
  return {
    ...stroke,
    points: stroke.points.map((point) => ({ ...point })),
  };
}

function cloneEraserStroke(stroke: EraserStroke): EraserStroke {
  return {
    ...stroke,
    points: stroke.points.map((point) => ({ ...point })),
  };
}

function cloneWandStroke(stroke: WandStroke): WandStroke {
  return {
    ...stroke,
    points: stroke.points.map((point) => ({ ...point })),
  };
}

function updateSelectedConfig(
  state: IconForgeState,
  updater: (config: ItemConfig) => ItemConfig,
): Partial<IconForgeState> {
  if (!state.selectedItemId) return {};
  const config = state.itemConfigs[state.selectedItemId];
  if (!config) return {};
  return {
    itemConfigs: {
      ...state.itemConfigs,
      [state.selectedItemId]: updater(config),
    },
  };
}

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
