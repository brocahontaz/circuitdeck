/**
 * Tests for the hash router.
 */
import { afterEach, describe, expect, it } from "vitest";
import { VIEWS, currentView, type ViewId } from "../src/lib/router";

describe("currentView", () => {
  afterEach(() => {
    window.location.hash = "";
  });

  it("defaults to overview for empty hash", () => {
    expect(currentView("")).toBe("overview");
    expect(currentView("#")).toBe("overview");
    expect(currentView("#/")).toBe("overview");
  });

  it("parses each known view", () => {
    for (const v of VIEWS) {
      expect(currentView(`#/${v.id}`)).toBe(v.id);
    }
  });

  it("falls back to overview for unknown ids", () => {
    expect(currentView("#/nonexistent")).toBe("overview");
    expect(currentView("#/../../etc")).toBe("overview");
  });
});

describe("VIEWS registry", () => {
  it("contains all eight required views", () => {
    const ids: ViewId[] = VIEWS.map((v) => v.id);
    expect(ids).toEqual([
      "overview",
      "production",
      "power",
      "factory",
      "logistics",
      "trains",
      "research",
      "platforms",
    ]);
  });
});
