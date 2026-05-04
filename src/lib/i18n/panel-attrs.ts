import type { PanelAttrConfig } from "$lib/settings-store";
import { t, type MessageKey } from "./index.svelte";

const PANEL_ATTR_MESSAGE_KEYS = {
  11720: "game.panelAttr.11720",
  11710: "game.panelAttr.11710",
  11930: "game.panelAttr.11930",
  11780: "game.panelAttr.11780",
  11940: "game.panelAttr.11940",
  11950: "game.panelAttr.11950",
  11760: "game.panelAttr.11760",
  11960: "game.panelAttr.11960",
  11010: "game.panelAttr.11010",
  11020: "game.panelAttr.11020",
  11030: "game.panelAttr.11030",
  11330: "game.panelAttr.11330",
  11340: "game.panelAttr.11340",
  11730: "game.panelAttr.11730",
  12510: "game.panelAttr.12510",
  12530: "game.panelAttr.12530",
  12540: "game.panelAttr.12540",
  11810: "game.panelAttr.11810",
  11970: "game.panelAttr.11970",
  11350: "game.panelAttr.11350",
  11500: "game.panelAttr.11500",
  11510: "game.panelAttr.11510",
  11520: "game.panelAttr.11520",
  11530: "game.panelAttr.11530",
  11540: "game.panelAttr.11540",
  11550: "game.panelAttr.11550",
  11560: "game.panelAttr.11560",
  11570: "game.panelAttr.11570",
  11580: "game.panelAttr.11580",
  13100: "game.panelAttr.13100",
  13110: "game.panelAttr.13110",
  13120: "game.panelAttr.13120",
  13130: "game.panelAttr.13130",
  13140: "game.panelAttr.13140",
  13150: "game.panelAttr.13150",
  13160: "game.panelAttr.13160",
  13170: "game.panelAttr.13170",
  13180: "game.panelAttr.13180",
} as const satisfies Record<number, MessageKey>;

export function resolvePanelAttrLabel(
  attr: Pick<PanelAttrConfig, "attrId" | "label">,
): string {
  const key = PANEL_ATTR_MESSAGE_KEYS[
    attr.attrId as keyof typeof PANEL_ATTR_MESSAGE_KEYS
  ];
  return key ? t(key) : attr.label;
}
