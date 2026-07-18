<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { t } from "$lib/i18n/index.svelte";

  let {
    open = $bindable(false),
    title,
    defaultValue = "",
    placeholder = "",
    onconfirm,
    oncancel,
  }: {
    open: boolean;
    title: string;
    defaultValue?: string;
    placeholder?: string;
    onconfirm: (value: string) => void;
    oncancel?: () => void;
  } = $props();

  let inputValue = $state(defaultValue);
  let inputRef: HTMLElement | null = $state(null);

  $effect(() => {
    if (open) {
      inputValue = defaultValue;
      queueMicrotask(() => {
        const el = inputRef as HTMLInputElement | null;
        el?.focus();
      });
    }
  });

  function handleConfirm() {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    onconfirm(trimmed);
    open = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      handleConfirm();
    }
    if (event.key === "Escape") {
      open = false;
      oncancel?.();
    }
  }

  function handleOpenChange(isOpen: boolean) {
    if (!isOpen) {
      oncancel?.();
    }
    open = isOpen;
  }
</script>

<Dialog.Root {open} onOpenChange={handleOpenChange}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
    </Dialog.Header>
    <div class="py-4">
      <Input
        bind:ref={inputRef}
        bind:value={inputValue}
        onkeydown={handleKeydown}
        {placeholder}
      />
    </div>
    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => {
          open = false;
          oncancel?.();
        }}
      >
        {t("common.cancel")}
      </Button>
      <Button onclick={handleConfirm}>
        {t("common.confirm")}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
