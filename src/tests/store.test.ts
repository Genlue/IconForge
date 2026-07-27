import { describe, it, expect, beforeEach } from "vitest";
import { useIconForgeStore } from "../store/useIconForgeStore";

describe("store", () => {
  beforeEach(() => {
    useIconForgeStore.setState({
      items: [],
      selectedItemId: null,
      error: null,
      lastExportResult: null,
    });
  });

  it("should start with empty items", () => {
    const state = useIconForgeStore.getState();
    expect(state.items).toEqual([]);
    expect(state.selectedItemId).toBeNull();
  });
});
