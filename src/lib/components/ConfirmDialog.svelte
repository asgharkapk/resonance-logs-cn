<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  let {
    open = $bindable(false),
    title,
    description,
    confirmText,
    cancelText,
    onconfirm,
    oncancel,
    variant = "destructive",
  }: {
    open: boolean;
    title: string;
    description?: string;
    confirmText: string;
    cancelText: string;
    onconfirm: () => void;
    oncancel?: () => void;
    variant?: "default" | "destructive";
  } = $props();

  function handleConfirm() {
    onconfirm();
    open = false;
  }

  function handleCancel() {
    oncancel?.();
    open = false;
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
      {#if description}
        <Dialog.Description>{description}</Dialog.Description>
      {/if}
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={handleCancel}>
        {cancelText}
      </Button>
      <Button {variant} onclick={handleConfirm}>
        {confirmText}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
