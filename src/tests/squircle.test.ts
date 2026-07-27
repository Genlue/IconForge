import { describe, it, expect } from "vitest";
import { sampleSquirclePath, pointsToSvgPath } from "../lib/squircle";

describe("squircle", () => {
  it("should sample 256 points by default", () => {
    const pts = sampleSquirclePath(4);
    expect(pts.length).toBe(256);
  });

  it("should generate a closed path", () => {
    const pts = sampleSquirclePath(4, 64);
    const path = pointsToSvgPath(pts);
    expect(path).toMatch(/^M /);
    expect(path).toMatch(/ Z$/);
  });
});
