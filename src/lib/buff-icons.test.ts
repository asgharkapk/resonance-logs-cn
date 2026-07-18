import { describe, expect, it } from "vitest";
import { ensureBuffIconOverrides, resolveBuffIconSrc } from "./buff-icons";

const DIR_PREFIX = "http://asset.localhost/C%3A%2Fapp%2Fbuff-icons%2F";

describe("ensureBuffIconOverrides", () => {
  it("keeps well-formed entries", () => {
    expect(
      ensureBuffIconOverrides({
        "997110": "997110_1a2b3c4d.png",
        "2110065": "2110065_000000ff.webp",
      }),
    ).toEqual({
      "997110": "997110_1a2b3c4d.png",
      "2110065": "2110065_000000ff.webp",
    });
  });

  it("drops malformed keys and unsafe file names", () => {
    expect(
      ensureBuffIconOverrides({
        abc: "997110_1a2b3c4d.png", // non-numeric key
        "997110": "../escape.png", // path traversal
        "997111": "997111_1A2B3C4D.png", // uppercase hex
        "997112": "997112_1a2b3c4d.bmp", // disallowed extension
        "997113": "997113_1a2b3c4.png", // short hash
        "997114": "", // empty value
      }),
    ).toEqual({});
  });

  it("tolerates null/undefined input", () => {
    expect(ensureBuffIconOverrides(null)).toEqual({});
    expect(ensureBuffIconOverrides(undefined)).toEqual({});
  });
});

describe("resolveBuffIconSrc", () => {
  const overrides = { "997110": "997110_1a2b3c4d.png" };

  it("prefers the player override over the game sprite", () => {
    expect(
      resolveBuffIconSrc(997110, "buff_default_1.png", overrides, DIR_PREFIX),
    ).toBe(`${DIR_PREFIX}997110_1a2b3c4d.png`);
  });

  it("gives icon-less buffs an asset URL when overridden", () => {
    expect(resolveBuffIconSrc(997110, null, overrides, DIR_PREFIX)).toBe(
      `${DIR_PREFIX}997110_1a2b3c4d.png`,
    );
  });

  it("falls back to the game sprite without an override", () => {
    expect(
      resolveBuffIconSrc(2110065, "buff_food_up_1.png", {}, DIR_PREFIX),
    ).toBe("/images/buff/buff_food_up_1.png");
  });

  it("falls back to the sprite while the icon dir is not ready", () => {
    expect(
      resolveBuffIconSrc(997110, "buff_default_1.png", overrides, null),
    ).toBe("/images/buff/buff_default_1.png");
    expect(resolveBuffIconSrc(997110, null, overrides, null)).toBeNull();
  });

  it("returns null when neither override nor sprite exists", () => {
    expect(resolveBuffIconSrc(123, null, {}, DIR_PREFIX)).toBeNull();
    expect(resolveBuffIconSrc(123, undefined, {}, DIR_PREFIX)).toBeNull();
  });
});
