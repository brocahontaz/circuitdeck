/**
 * Tests for the format helpers.
 */
import { describe, expect, it } from "vitest";
import {
  fmtInt,
  fmtNumber,
  fmtPerMin,
  fmtPct,
  fmtTicks,
} from "../src/lib/format";

describe("fmtNumber", () => {
  it("renders dash for null/undefined/NaN", () => {
    expect(fmtNumber(null)).toBe("—");
    expect(fmtNumber(undefined)).toBe("—");
    expect(fmtNumber(Number.NaN)).toBe("—");
  });

  it("formats small numbers with digits", () => {
    expect(fmtNumber(3.14159, 2)).toBe("3.14");
  });

  it("abbreviates thousands and millions", () => {
    expect(fmtNumber(15000)).toBe("15.0k");
    expect(fmtNumber(2400000)).toBe("2.4M");
  });
});

describe("fmtInt", () => {
  it("rounds and groups", () => {
    expect(fmtInt(1234.6)).toBe("1,235");
  });

  it("renders dash for null", () => {
    expect(fmtInt(null)).toBe("—");
  });
});

describe("fmtPct", () => {
  it("formats percent without decimals by default", () => {
    expect(fmtPct(42.4)).toBe("42%");
  });

  it("renders dash for null", () => {
    expect(fmtPct(null)).toBe("—");
  });
});

describe("fmtPerMin", () => {
  it("appends /min", () => {
    expect(fmtPerMin(12.34, 1)).toBe("12.3/min");
  });

  it("renders dash for null", () => {
    expect(fmtPerMin(null)).toBe("—");
  });
});

describe("fmtTicks", () => {
  it("formats under an hour in minutes", () => {
    expect(fmtTicks(60 * 60 * 5)).toBe("5m");
  });

  it("formats hours", () => {
    expect(fmtTicks(60 * 60 * 90)).toBe("1h 30m");
  });

  it("renders dash for null", () => {
    expect(fmtTicks(null)).toBe("—");
  });
});
