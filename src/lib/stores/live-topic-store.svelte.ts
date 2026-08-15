import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type LiveTopicStatus =
  | { state: "idle" }
  | { state: "loading" }
  | { state: "ready" }
  | { state: "error"; message: string };

type Revisioned = { revision: number };

/**
 * Generic replace-only store for one live publication topic.
 * Ignores out-of-order revisions; consumers connect/disconnect by refcount.
 * When the last consumer disconnects, `data` is cleared so a later reconnect
 * never renders one stale frame before the bootstrap lands.
 */
export class LiveTopicStore<T extends Revisioned> {
  data = $state.raw<T | null>(null);
  status = $state.raw<LiveTopicStatus>({ state: "idle" });

  #eventName: string;
  #bootstrap: () => Promise<
    { status: "ok"; data: T } | { status: "error"; error: string }
  >;
  #connectPromise: Promise<void> | null = null;
  #unlisten: UnlistenFn | null = null;
  #consumers = 0;

  constructor(
    eventName: string,
    bootstrap: () => Promise<
      { status: "ok"; data: T } | { status: "error"; error: string }
    >,
  ) {
    this.#eventName = eventName;
    this.#bootstrap = bootstrap;
  }

  get topicLabel(): string {
    return this.#eventName;
  }

  async connect(): Promise<() => void> {
    this.#consumers += 1;
    try {
      if (!this.#connectPromise) {
        this.status = { state: "loading" };
        this.#connectPromise = this.#connect();
      }
      await this.#connectPromise;
    } catch (error) {
      this.#consumers -= 1;
      throw error;
    }

    let released = false;
    return () => {
      if (released) return;
      released = true;
      this.#consumers -= 1;
      if (this.#consumers > 0) return;

      this.#unlisten?.();
      this.#unlisten = null;
      this.#connectPromise = null;
      this.data = null;
      this.status = { state: "idle" };
    };
  }

  apply(next: T) {
    if (this.data && next.revision < this.data.revision) return;
    this.data = next;
    this.status = { state: "ready" };
  }

  async #connect(): Promise<void> {
    try {
      this.#unlisten = await listen<T>(this.#eventName, (event) => {
        this.apply(event.payload);
      });

      const result = await this.#bootstrap();
      if (result.status === "error") throw new Error(result.error);
      this.apply(result.data);
    } catch (error) {
      this.#unlisten?.();
      this.#unlisten = null;
      this.#connectPromise = null;
      this.status = {
        state: "error",
        message: error instanceof Error ? error.message : String(error),
      };
      throw error;
    }
  }
}

/**
 * Connects several topic stores and returns one disposer. Individual connect
 * failures are logged instead of failing the whole window. Disconnects that
 * happen before the async connect resolves are handled by checking the
 * disposer state at resolve time.
 */
export function connectTopics(
  ...stores: LiveTopicStore<Revisioned>[]
): () => void {
  let disposed = false;
  const disconnects: Array<() => void> = [];
  for (const store of stores) {
    void store
      .connect()
      .then((disconnect) => {
        if (disposed) disconnect();
        else disconnects.push(disconnect);
      })
      .catch((error) => {
        console.error(
          `Failed to connect live topic "${store.topicLabel}"`,
          error,
        );
      });
  }
  return () => {
    disposed = true;
    for (const disconnect of disconnects.splice(0)) disconnect();
  };
}
