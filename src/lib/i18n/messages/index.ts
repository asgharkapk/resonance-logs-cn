import type { AppLocale } from "../locales";
import { enUSMessages } from "./en-US";
import { jaJPMessages } from "./ja-JP";
import { zhCNMessages } from "./zh-CN";

export type MessageKey = keyof typeof zhCNMessages;
export type MessageCatalog = Partial<Record<MessageKey, string>>;

export const MESSAGES = {
  "zh-CN": zhCNMessages,
  "en-US": enUSMessages,
  "ja-JP": jaJPMessages,
} satisfies Record<AppLocale, MessageCatalog>;
