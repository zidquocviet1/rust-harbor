<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { Code2, ChevronDown, Loader2 } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import * as si from "simple-icons";

  interface Editor {
    id: string;
    name: string;
    icon: string;
  }

  let {
    repoPath,
    variant = "outline",
    size = "sm",
    class: className = "",
    externalEditors = []
  } = $props<{
    repoPath: string;
    variant?: "outline" | "ghost" | "default";
    size?: "sm" | "default" | "icon";
    class?: string;
    externalEditors?: Editor[];
  }>();

  let editors = $derived(externalEditors);
  let isOpen = $state(false);
  let isLoading = $state(false);
  let buttonEl: HTMLButtonElement | null = $state(null);
  let dropdownStyle = $state("");

  async function openInEditor(editorId: string) {
    try {
      await invoke("open_in_editor", { editorId, path: repoPath });
      toast.success("Opening editor...");
      isOpen = false;
    } catch (e) {
      toast.error(`Failed: ${e}`);
    }
  }

  function toggleDropdown(e: MouseEvent) {
    e.stopPropagation();
    e.preventDefault();

    if (isOpen) {
      isOpen = false;
      return;
    }

    if (buttonEl) {
      const rect = buttonEl.getBoundingClientRect();
      const dropdownWidth = 208;

      let left = rect.right - dropdownWidth;
      if (left < 10) left = 10;

      dropdownStyle = `position: fixed; top: ${rect.bottom + 4}px; left: ${left}px;`;
    }

    isOpen = true;
  }

  function getIcon(iconName: string) {
    const key = `si${iconName.charAt(0).toUpperCase()}${iconName.slice(1).toLowerCase()}` as keyof typeof si;
    const icon = si[key];
    if (icon) return icon.svg;
    return null;
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.open-in-editor-container') && !target.closest('.open-in-editor-dropdown')) {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      setTimeout(() => {
        document.addEventListener('click', handleClickOutside);
      }, 10);
    }

    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });

  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      }
    };
  }
</script>

<div class="relative open-in-editor-container">
  <Button
    bind:ref={buttonEl}
    {variant}
    {size}
    class="flex items-center gap-1.5 {className}"
    onclick={toggleDropdown}
    disabled={isLoading || editors.length === 0}
  >
    {#if isLoading}
      <Loader2 class="w-3.5 h-3.5 animate-spin" />
    {:else}
      <Code2 class="w-3.5 h-3.5" />
    {/if}
    <span class="hidden sm:inline">Open In</span>
    <ChevronDown class="w-3 h-3 opacity-50" />
  </Button>
</div>

{#if isOpen && editors.length > 0}
  <div
    use:portal
    class="open-in-editor-dropdown w-52 bg-white border border-slate-200 rounded-xl shadow-2xl py-2"
    style="{dropdownStyle} z-index: 99999;"
  >
    <div class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400">
      Choose Editor
    </div>
    {#each editors as editor}
      <button
        class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-100 text-left transition-colors group"
        onclick={() => openInEditor(editor.id)}
      >
        <div class="w-5 h-5 flex items-center justify-center opacity-70 group-hover:opacity-100">
          {#if getIcon(editor.icon)}
            <div class="w-4 h-4 fill-current">
              {@html getIcon(editor.icon)}
            </div>
          {:else}
            <Code2 class="w-4 h-4" />
          {/if}
        </div>
        <span class="text-sm font-medium text-slate-700 group-hover:text-primary transition-colors">{editor.name}</span>
      </button>
    {/each}
  </div>
{/if}
