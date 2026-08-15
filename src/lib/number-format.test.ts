import { describe, expect, it } from "vitest";
import { abbreviateNumber, abbreviateNumberSplit } from "./number-format";

describe("abbreviateNumberSplit", () => {
  it("uses k/m/b/t for the western style", () => {
    expect(abbreviateNumberSplit(999, 1, "western")).toEqual(["999", ""]);
    expect(abbreviateNumberSplit(1_500, 1, "western")).toEqual(["1.5", "k"]);
    expect(abbreviateNumberSplit(2_400_000, 1, "western")).toEqual([
      "2.4",
      "m",
    ]);
    expect(abbreviateNumberSplit(3_000_000_000, 2, "western")).toEqual([
      "3.00",
      "b",
    ]);
    expect(abbreviateNumberSplit(5e12, 1, "western")).toEqual(["5.0", "t"]);
  });

  it("switches to 万/亿 magnitudes for the cn style", () => {
    expect(abbreviateNumberSplit(9_999, 1, "cn")).toEqual(["9999", ""]);
    expect(abbreviateNumberSplit(7_713_800, 2, "cn")).toEqual(["771.38", "w"]);
    expect(abbreviateNumberSplit(2.5e8, 1, "cn")).toEqual(["2.5", "亿"]);
  });
});

describe("abbreviateNumber", () => {
  it("joins the mantissa and suffix for plain-text contexts", () => {
    expect(abbreviateNumber(1_500, 1, "western")).toBe("1.5k");
    expect(abbreviateNumber(7_713_800, 2, "cn")).toBe("771.38w");
    expect(abbreviateNumber(0, 1, "western")).toBe("0");
  });
});
