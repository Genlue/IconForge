import { describe, it, expect, beforeEach } from "vitest";
import { useIconForgeStore } from "../store/useIconForgeStore";
import { DEFAULT_RENDER_CONFIG, DEFAULT_UPSCALE_CONFIG } from "../constants/defaults";
import { ExportMode, InputFileType } from "../types/domain";
import type { InputItem, ItemConfig } from "../types/domain";

function makeItem(id: string): InputItem {
  return {
    id,
    sourcePath: `C:\\icons\\${id}.png`,
    displayName: `${id}.png`,
    fileType: InputFileType.Image,
    parentDirectoryPath: "C:\\icons",
    resolvedTargetPath: null,
    sourceWidth: 256,
    sourceHeight: 256,
    thumbnailPngBase64: "",
    supportedExportModes: [ExportMode.ExportAsIco],
    warnings: [],
  };
}

function makeConfig(): ItemConfig {
  return {
    renderConfig: {
      ...DEFAULT_RENDER_CONFIG,
      outerShadow: { ...DEFAULT_RENDER_CONFIG.outerShadow },
      stroke: { ...DEFAULT_RENDER_CONFIG.stroke },
    },
    upscaleConfig: { ...DEFAULT_UPSCALE_CONFIG },
    activePresetId: "macos-classic-rounded",
  };
}

describe("store", () => {
  beforeEach(() => {
    useIconForgeStore.setState({
      items: [],
      selectedItemId: null,
      itemConfigs: {},
      error: null,
      lastExportResult: null,
    });
  });

  it("should start with empty items", () => {
    const state = useIconForgeStore.getState();
    expect(state.items).toEqual([]);
    expect(state.selectedItemId).toBeNull();
  });

  it("edits only the selected icon", () => {
    const first = makeItem("first");
    const second = makeItem("second");
    useIconForgeStore.setState({
      items: [first, second],
      selectedItemId: first.id,
      itemConfigs: { first: makeConfig(), second: makeConfig() },
    });

    useIconForgeStore.getState().updateRenderConfig({ canvasInset: 24 });
    useIconForgeStore.getState().updateUpscaleConfig({ enabled: true });

    const state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.renderConfig.canvasInset).toBe(24);
    expect(state.itemConfigs.first?.upscaleConfig.enabled).toBe(true);
    expect(state.itemConfigs.second?.renderConfig.canvasInset).toBe(0);
    expect(state.itemConfigs.second?.upscaleConfig.enabled).toBe(false);
  });

  it("copies the current config to every icon without sharing nested objects", () => {
    const first = makeItem("first");
    const second = makeItem("second");
    useIconForgeStore.setState({
      items: [first, second],
      selectedItemId: first.id,
      itemConfigs: { first: makeConfig(), second: makeConfig() },
    });
    useIconForgeStore.getState().updateOuterShadow({ enabled: true, blurRadius: 18 });
    useIconForgeStore.getState().updateStroke({ width: 3 });
    useIconForgeStore.getState().updateUpscaleConfig({ enabled: true, denoise: 2 });

    useIconForgeStore.getState().applyCurrentConfigToAll();
    let state = useIconForgeStore.getState();
    expect(state.itemConfigs.second).toEqual(state.itemConfigs.first);
    expect(state.itemConfigs.second).not.toBe(state.itemConfigs.first);
    expect(state.itemConfigs.second?.renderConfig.outerShadow).not.toBe(
      state.itemConfigs.first?.renderConfig.outerShadow,
    );

    useIconForgeStore.getState().selectItem(second.id);
    useIconForgeStore.getState().updateOuterShadow({ blurRadius: 4 });
    state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.renderConfig.outerShadow.blurRadius).toBe(18);
    expect(state.itemConfigs.second?.renderConfig.outerShadow.blurRadius).toBe(4);
  });
});
