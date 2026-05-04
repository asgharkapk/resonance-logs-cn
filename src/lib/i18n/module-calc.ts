import type { ModuleInfo, ModulePart } from "$lib/api";
import { t, type MessageKey } from "./index.svelte";

export type ModuleAttrOption = {
  id: number;
  label: string;
};

export type ModuleAttrDisplayEntry = ModuleAttrOption & {
  value: number;
};

export const MODULE_ATTR_IDS = [
  1110, 1111, 1112, 1113, 1114, 1205, 1206, 1407, 1408, 1409, 1410, 1307, 1308,
  2104, 2105, 2204, 2205, 2404, 2405, 2406, 2304,
] as const;

const MODULE_ATTR_MESSAGE_KEYS = {
  1110: "game.moduleAttr.1110",
  1111: "game.moduleAttr.1111",
  1112: "game.moduleAttr.1112",
  1113: "game.moduleAttr.1113",
  1114: "game.moduleAttr.1114",
  1205: "game.moduleAttr.1205",
  1206: "game.moduleAttr.1206",
  1407: "game.moduleAttr.1407",
  1408: "game.moduleAttr.1408",
  1409: "game.moduleAttr.1409",
  1410: "game.moduleAttr.1410",
  1307: "game.moduleAttr.1307",
  1308: "game.moduleAttr.1308",
  2104: "game.moduleAttr.2104",
  2105: "game.moduleAttr.2105",
  2204: "game.moduleAttr.2204",
  2205: "game.moduleAttr.2205",
  2404: "game.moduleAttr.2404",
  2405: "game.moduleAttr.2405",
  2406: "game.moduleAttr.2406",
  2304: "game.moduleAttr.2304",
} as const satisfies Record<number, MessageKey>;

const MODULE_NAME_MESSAGE_KEYS = {
  5500101: "game.moduleName.5500101",
  5500102: "game.moduleName.5500102",
  5500103: "game.moduleName.5500103",
  5500104: "game.moduleName.5500104",
  5500201: "game.moduleName.5500201",
  5500202: "game.moduleName.5500202",
  5500203: "game.moduleName.5500203",
  5500204: "game.moduleName.5500204",
  5500301: "game.moduleName.5500301",
  5500302: "game.moduleName.5500302",
  5500303: "game.moduleName.5500303",
  5500304: "game.moduleName.5500304",
} as const satisfies Record<number, MessageKey>;

export function resolveModuleAttrLabel(
  attrId: number | null | undefined,
  fallback?: string,
): string {
  const key =
    attrId === null || attrId === undefined
      ? undefined
      : MODULE_ATTR_MESSAGE_KEYS[
          attrId as keyof typeof MODULE_ATTR_MESSAGE_KEYS
        ];

  if (key) return t(key);
  if (fallback) return fallback;
  return t("moduleCalc.unknownAttribute", { id: attrId ?? "?" });
}

export function resolveModuleName(
  configId: number | null | undefined,
  fallback?: string,
): string {
  const key =
    configId === null || configId === undefined
      ? undefined
      : MODULE_NAME_MESSAGE_KEYS[
          configId as keyof typeof MODULE_NAME_MESSAGE_KEYS
        ];

  if (key) return t(key);
  if (fallback) return fallback;
  return t("moduleCalc.unknownModule", { id: configId ?? "?" });
}

export function getModuleAttrOptions(): ModuleAttrOption[] {
  return MODULE_ATTR_IDS.map((id) => ({
    id,
    label: resolveModuleAttrLabel(id),
  }));
}

export function buildModuleAttrDisplayEntries(
  modules: Array<Pick<ModuleInfo, "parts">>,
): ModuleAttrDisplayEntry[] {
  const totals = new Map<number, { value: number; fallback?: string }>();

  for (const module of modules) {
    for (const part of module.parts) {
      const entry = totals.get(part.id);
      if (entry) {
        entry.value += part.value;
        entry.fallback ??= part.name;
      } else {
        totals.set(part.id, { value: part.value, fallback: part.name });
      }
    }
  }

  return [...totals.entries()].map(([id, entry]) => ({
    id,
    label: resolveModuleAttrLabel(id, entry.fallback),
    value: entry.value,
  }));
}

export function resolveModulePartLabel(part: Pick<ModulePart, "id" | "name">) {
  return resolveModuleAttrLabel(part.id, part.name);
}
