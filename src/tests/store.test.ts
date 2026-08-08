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
      gloss: { ...DEFAULT_RENDER_CONFIG.gloss },
      autoCutout: { ...DEFAULT_RENDER_CONFIG.autoCutout },
      hqRender: { ...DEFAULT_RENDER_CONFIG.hqRender },
    },
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
    useIconForgeStore.getState().addBrushStroke({
      points: [{ x: 10, y: 10 }, { x: 30, y: 30 }],
      color: "#FF0000FF",
      size: 8,
      opacity: 1,
      mode: "paint",
      clipToMask: true,
    });

    useIconForgeStore.getState().applyCurrentConfigToAll();
    let state = useIconForgeStore.getState();
    expect(state.itemConfigs.second?.renderConfig).toEqual(state.itemConfigs.first?.renderConfig);
    expect(state.itemConfigs.second?.upscaleConfig).toEqual(state.itemConfigs.first?.upscaleConfig);
    expect(state.itemConfigs.second).not.toBe(state.itemConfigs.first);
    expect(state.itemConfigs.second?.renderConfig.outerShadow).not.toBe(
      state.itemConfigs.first?.renderConfig.outerShadow,
    );
    expect(state.itemConfigs.first?.brushStrokes).toHaveLength(1);
    expect(state.itemConfigs.second?.brushStrokes).toHaveLength(1);

    useIconForgeStore.getState().selectItem(second.id);
    useIconForgeStore.getState().updateOuterShadow({ blurRadius: 4 });
    state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.renderConfig.outerShadow.blurRadius).toBe(18);
    expect(state.itemConfigs.second?.renderConfig.outerShadow.blurRadius).toBe(4);
  });

  it("applies a preview-picked color to the active color field", () => {
    const item = makeItem("first");
    useIconForgeStore.setState({
      items: [item],
      selectedItemId: item.id,
      itemConfigs: { first: makeConfig() },
    });

    useIconForgeStore.getState().startColorPicking("backplate");
    useIconForgeStore.getState().applyPickedColor("#12ABEF");

    const state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.renderConfig.backplateColor).toBe("#12AB EFFF".replace(" ", ""));
    expect(state.previewTool).toBe("none");
    expect(state.colorPickTarget).toBeNull();
  });

  it("updates hq render parameters only for the selected icon", () => {
    const first = makeItem("first");
    const second = makeItem("second");
    useIconForgeStore.setState({
      items: [first, second],
      selectedItemId: first.id,
      itemConfigs: { first: makeConfig(), second: makeConfig() },
    });

    useIconForgeStore.getState().updateHqRender({ bg: 0.42, thresh: 0.8, enabled: true });

    const state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.renderConfig.hqRender.bg).toBe(0.42);
    expect(state.itemConfigs.first?.renderConfig.hqRender.thresh).toBe(0.8);
    expect(state.itemConfigs.first?.renderConfig.hqRender.enabled).toBe(true);
    expect(state.itemConfigs.second?.renderConfig.hqRender.bg).toBe(0.1);
    expect(state.itemConfigs.second?.renderConfig.hqRender.enabled).toBe(false);
  });

  it("clone on apply-to-all does not share hq render config", () => {
    const first = makeItem("first");
    const second = makeItem("second");
    useIconForgeStore.setState({
      items: [first, second],
      selectedItemId: first.id,
      itemConfigs: { first: makeConfig(), second: makeConfig() },
    });
    useIconForgeStore.getState().updateHqRender({ bg: 0.77 });
    useIconForgeStore.getState().applyCurrentConfigToAll();

    const state = useIconForgeStore.getState();
    expect(state.itemConfigs.second?.renderConfig).toEqual(
      state.itemConfigs.first?.renderConfig,
    );
    expect(state.itemConfigs.second?.renderConfig.hqRender).not.toBe(
      state.itemConfigs.first?.renderConfig.hqRender,
    );
  });

  it("tracks magic wand strokes with undo and clear", () => {
    const item = makeItem("first");
    useIconForgeStore.setState({
      items: [item],
      selectedItemId: item.id,
      itemConfigs: { first: makeConfig() },
    });

    const strokeA = { points: [{ x: 30, y: 40 }], tolerance: 65 };
    const strokeB = { points: [{ x: 90, y: 10 }], tolerance: 20 };
    useIconForgeStore.getState().addWandStroke(strokeA);
    useIconForgeStore.getState().addWandStroke(strokeB);
    useIconForgeStore.getState().setWandTolerance(50);

    let state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.wandStrokes).toHaveLength(2);
    expect(state.itemConfigs.first?.wandStrokes[0]?.tolerance).toBe(65);
    expect(state.itemConfigs.first?.wandTolerance).toBe(50);

    useIconForgeStore.getState().undoWandStroke();
    state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.wandStrokes).toHaveLength(1);
    expect(state.itemConfigs.first?.wandStrokes[0]?.points).toEqual([{ x: 30, y: 40 }]);

    useIconForgeStore.getState().clearWandStrokes();
    state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.wandStrokes).toHaveLength(0);
  });

  it("delete key flow: selection converts into a wand stroke", () => {
    const item = makeItem("first");
    useIconForgeStore.setState({
      items: [item],
      selectedItemId: item.id,
      previewTool: "magic-wand",
      itemConfigs: { first: makeConfig() },
    });

    useIconForgeStore.getState().setWandSelection({
      x: 40,
      y: 60,
      tolerance: 30,
      maskPngBase64: "mask",
    });
    useIconForgeStore.getState().deleteWandSelection();

    const state = useIconForgeStore.getState();
    expect(state.wandSelection).toBeNull();
    expect(state.itemConfigs.first?.wandStrokes).toEqual([
      { points: [{ x: 40, y: 60 }], tolerance: 30 },
    ]);
  });

  it("tracks source-eraser strokes with undo and clear", () => {
    const item = makeItem("first");
    useIconForgeStore.setState({
      items: [item],
      selectedItemId: item.id,
      itemConfigs: { first: makeConfig() },
    });

    useIconForgeStore.getState().addEraserStroke({
      points: [{ x: 10, y: 20 }, { x: 50, y: 50 }],
      size: 20,
      hardness: 70,
    });
    useIconForgeStore.getState().updateEraserSettings({ size: 8, hardness: 30 });

    let state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.eraserStrokes).toHaveLength(1);
    expect(state.itemConfigs.first?.eraserStrokes[0]?.size).toBe(20);
    expect(state.itemConfigs.first?.eraserSize).toBe(8);
    expect(state.itemConfigs.first?.eraserHardness).toBe(30);

    useIconForgeStore.getState().undoEraserStroke();
    useIconForgeStore.getState().clearEraserStrokes();
    state = useIconForgeStore.getState();
    expect(state.itemConfigs.first?.eraserStrokes).toHaveLength(0);
  });

  it("saves, renames and deletes custom presets without the source-processing bits", () => {
    const item = makeItem("first");
    useIconForgeStore.setState({
      items: [item],
      selectedItemId: item.id,
      itemConfigs: { first: makeConfig() },
    });

    useIconForgeStore
      .getState()
      .updateRenderConfig({ autoCutout: { enabled: true, tolerance: 55, feather: 6 }, canvasInset: 42 });
    const presetId = useIconForgeStore.getState().saveCurrentAsPreset();
    expect(presetId).toBeTruthy();

    let state = useIconForgeStore.getState();
    const saved = state.customPresets[0];
    expect(saved?.name).toMatch(/^自定义预设 /);
    expect(saved?.config.canvasInset).toBe(42);
    // Auto cutout (原图处理) is excluded from saved presets.
    expect(saved?.config.autoCutout.enabled).toBe(false);
    expect(state.itemConfigs.first?.activePresetId).toBe(presetId);

    useIconForgeStore.getState().renamePreset(presetId!, "我的风格");
    state = useIconForgeStore.getState();
    expect(state.customPresets[0]?.name).toBe("我的风格");

    // Custom preset can be applied.
    useIconForgeStore.getState().applyPreset(presetId!);
    expect(useIconForgeStore.getState().itemConfigs.first?.renderConfig.canvasInset).toBe(42);

    useIconForgeStore.getState().deletePreset(presetId!);
    state = useIconForgeStore.getState();
    expect(state.customPresets).toHaveLength(0);
    expect(state.itemConfigs.first?.activePresetId).toBeNull();
  });
});
