<script lang="ts">
  import ProviderIcon from "$lib/components/icons/ProviderIcon.svelte";
  import { Check, ChevronDown } from "lucide-svelte";

  export interface ModelItem {
    provider: string;
    model: string;
  }

  interface Props {
    items: ModelItem[];
    value: string; // selected model
    valueProvider: string; // provider of selected item
    onchange: (provider: string, model: string) => void;
    disabled?: boolean;
  }

  let {
    items,
    value,
    valueProvider,
    onchange,
    disabled = false,
  }: Props = $props();

  let isOpen = $state(false);
  let buttonEl = $state<HTMLButtonElement | null>(null);
  let dropdownStyle = $state("");

  const PROVIDER_LABELS: Record<string, string> = {
    claude: "Claude",
    openai: "OpenAI",
    gemini: "Gemini",
    grok: "Grok",
    ollama: "Ollama",
  };

  // Group items by provider for section headers when multiple providers present
  const grouped = $derived(
    (): { provider: string; label: string; models: ModelItem[] }[] => {
      const map = new Map<string, ModelItem[]>();
      for (const item of items) {
        if (!map.has(item.provider)) map.set(item.provider, []);
        map.get(item.provider)!.push(item);
      }
      return Array.from(map.entries()).map(([p, ms]) => ({
        provider: p,
        label: PROVIDER_LABELS[p] ?? p,
        models: ms,
      }));
    },
  );

  const multiProvider = $derived(grouped().length > 1);

  function toggle(e: MouseEvent) {
    e.stopPropagation();
    if (disabled) return;
    if (isOpen) {
      isOpen = false;
      return;
    }
    if (buttonEl) {
      const rect = buttonEl.getBoundingClientRect();
      const dropdownWidth = Math.max(rect.width, 220);
      let left = rect.right - dropdownWidth;
      if (left < 8) left = 8;
      dropdownStyle = `position: fixed; top: ${rect.bottom + 4}px; left: ${left}px; width: ${dropdownWidth}px;`;
    }
    isOpen = true;
  }

  function select(provider: string, model: string) {
    onchange(provider, model);
    isOpen = false;
  }

  function handleClickOutside(e: MouseEvent) {
    const t = e.target as HTMLElement;
    if (
      !t.closest(".model-select-trigger") &&
      !t.closest(".model-select-dropdown")
    )
      isOpen = false;
  }

  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  }

  $effect(() => {
    if (isOpen)
      setTimeout(
        () => document.addEventListener("click", handleClickOutside),
        10,
      );
    return () => document.removeEventListener("click", handleClickOutside);
  });
</script>

<!-- Trigger button -->
<div class="relative model-select-trigger w-full">
  <button
    bind:this={buttonEl}
    type="button"
    onclick={toggle}
    {disabled}
    class="w-full flex items-center gap-2 bg-white/80 border border-slate-200/80 rounded-xl pl-2.5 pr-2 py-2 text-[12px] font-medium cursor-pointer
           focus:outline-none focus:ring-2 focus:ring-primary/40 focus:bg-white transition-all
           hover:bg-white hover:border-slate-300/80
           {disabled ? 'opacity-50 cursor-default pointer-events-none' : ''}"
  >
    {#if valueProvider}
      <ProviderIcon provider={valueProvider} />
    {/if}
    <span class="flex-1 text-left font-mono text-foreground truncate"
      >{value || "Select model…"}</span
    >
    <ChevronDown
      size={12}
      class="shrink-0 text-muted-foreground transition-transform duration-150 {isOpen
        ? 'rotate-180'
        : ''}"
    />
  </button>
</div>

<!-- Portal dropdown -->
{#if isOpen}
  <div
    use:portal
    class="model-select-dropdown bg-white border border-slate-200 rounded-xl shadow-2xl py-2 z-[99999]"
    style={dropdownStyle}
  >
    {#each grouped() as group}
      {#if multiProvider}
        <div
          class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400"
        >
          {group.label}
        </div>
      {/if}
      {#each group.models as item}
        {@const isSelected =
          item.model === value && item.provider === valueProvider}
        <button
          type="button"
          class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-50 text-left transition-colors group"
          onclick={() => select(item.provider, item.model)}
        >
          <div class="w-5 h-5 flex items-center justify-center shrink-0">
            {#if isSelected}
              <Check size={13} class="text-primary" />
            {/if}
          </div>
          <ProviderIcon provider={item.provider} />
          <span
            class="font-mono text-[12px] flex-1 truncate {isSelected
              ? 'text-primary font-semibold'
              : 'text-slate-700 group-hover:text-primary'} transition-colors"
          >
            {item.model}
          </span>
        </button>
      {/each}
    {/each}
  </div>
{/if}
