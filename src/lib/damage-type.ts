import { t, type MessageKey } from "./i18n/index.svelte";

export const NO_DAMAGE_INFO = "-" as const;

export const DAMAGE_PROPERTY_LABEL_KEYS: Readonly<Record<number, MessageKey>> =
  {
    0: "game.damageProperty.generic",
    1: "game.damageProperty.fire",
    2: "game.damageProperty.water",
    3: "game.damageProperty.thunder",
    4: "game.damageProperty.wood",
    5: "game.damageProperty.wind",
    6: "game.damageProperty.rock",
    7: "game.damageProperty.light",
    8: "game.damageProperty.dark",
  };

export const DAMAGE_MODE_LABEL_KEYS: Readonly<Record<number, MessageKey>> = {
  1: "game.damageMode.physical",
  2: "game.damageMode.magical",
};

export function propertyLabel(value: number | null | undefined): string {
  if (value == null) return NO_DAMAGE_INFO;
  const labelKey = DAMAGE_PROPERTY_LABEL_KEYS[value];
  return labelKey ? t(labelKey) : NO_DAMAGE_INFO;
}

export function damageModeLabel(value: number | null | undefined): string {
  if (value == null) return NO_DAMAGE_INFO;
  const labelKey = DAMAGE_MODE_LABEL_KEYS[value];
  return labelKey ? t(labelKey) : NO_DAMAGE_INFO;
}
