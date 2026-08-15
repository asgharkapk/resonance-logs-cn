import { describe, expect, it } from "vitest";
import { APP_LOCALES } from "$lib/i18n/locales";
import { presetSourceForLocale } from "./voice-preset-source";

describe("presetSourceForLocale", () => {
  it("maps every supported UI locale to a preset source carrying that locale", () => {
    for (const locale of APP_LOCALES) {
      expect(presetSourceForLocale(locale)).toEqual({
        mode: "preset",
        locale,
      });
    }
  });
});
