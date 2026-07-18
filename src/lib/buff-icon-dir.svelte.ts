import { convertFileSrc } from "@tauri-apps/api/core";
import { commands } from "$lib/bindings";

/**
 * Runtime handle for the buff-icon directory: the asset-protocol URL prefix
 * (`convertFileSrc` output, trailing slash) that `resolveBuffIconSrc`
 * concatenates override file names onto. Initialized once per window via
 * `initBuffIconDir`; stays `null` until the backend answers.
 */
let _iconDirUrlPrefix = $state<string | null>(null);
let _initPromise: Promise<void> | null = null;

export function buffIconDirUrlPrefix(): string | null {
  return _iconDirUrlPrefix;
}

export function initBuffIconDir(): Promise<void> {
  _initPromise ??= (async () => {
    const result = await commands.buffIconDir();
    if (result.status === "ok") {
      _iconDirUrlPrefix = convertFileSrc(result.data);
    } else {
      console.error("failed to resolve buff icon dir:", result.error);
    }
  })();
  return _initPromise;
}
