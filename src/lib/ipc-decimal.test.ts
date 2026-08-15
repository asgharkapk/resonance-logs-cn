import { describe, expect, it } from "vitest";
import {
  ipcAdd,
  ipcBigInt,
  ipcCompare,
  ipcIsZero,
  ipcRatio,
  ipcSum,
} from "./ipc-decimal";

describe("IPC decimal helpers", () => {
  it("normalizes invalid values and preserves integer strings", () => {
    expect(ipcBigInt("9007199254740993")).toBe(9_007_199_254_740_993n);
    expect(ipcBigInt("not-a-number")).toBe(0n);
    expect(ipcBigInt(null)).toBe(0n);
    expect(ipcIsZero("0")).toBe(true);
  });

  it("adds and compares values above the JS safe integer range exactly", () => {
    expect(ipcAdd("9007199254740993", "17")).toBe(9_007_199_254_741_010n);
    expect(ipcSum(["9007199254740993", 17n, "-10"])).toBe(
      9_007_199_254_741_000n,
    );
    expect(ipcCompare("9007199254740993", "9007199254740992")).toBe(1);
  });

  it("computes finite ratios without converting operands to Number first", () => {
    const unit = "9".repeat(320);
    expect(ipcRatio(unit, unit)).toBe(1);
    expect(ipcRatio(unit, `${unit}0`, 100)).toBe(10);
  });
});
