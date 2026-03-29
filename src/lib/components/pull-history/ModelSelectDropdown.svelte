<script lang="ts">
  import { Check, ChevronDown } from "lucide-svelte";

  interface Props {
    provider: string;
    models: string[];
    value: string;
    onchange: (model: string) => void;
    disabled?: boolean;
  }

  let { provider, models, value, onchange, disabled = false }: Props = $props();

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

  function toggle(e: MouseEvent) {
    e.stopPropagation();
    if (disabled) return;
    if (isOpen) {
      isOpen = false;
      return;
    }
    if (buttonEl) {
      const rect = buttonEl.getBoundingClientRect();
      const dropdownWidth = 252;
      let left = rect.left;
      if (left + dropdownWidth > window.innerWidth - 8)
        left = window.innerWidth - dropdownWidth - 8;
      dropdownStyle = `position: fixed; top: ${rect.bottom + 4}px; left: ${left}px; width: ${dropdownWidth}px;`;
    }
    isOpen = true;
  }

  function select(model: string) {
    onchange(model);
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

<!-- Provider icon snippet — distinct SVG emblem per provider -->
{#snippet providerIcon(prov: string)}
  {#if prov === "claude"}
    <span
      class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-amber-500 text-white shrink-0"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
        <polygon points="5,1 9.3,8.5 0.7,8.5" />
      </svg>
    </span>
  {:else if prov === "openai"}
    <span
      class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-slate-800 text-white shrink-0"
    >
      <svg
        width="10"
        height="10"
        viewBox="0 0 10 10"
        stroke="currentColor"
        stroke-width="1.4"
        stroke-linecap="round"
        fill="none"
      >
        <line x1="5" y1="1" x2="5" y2="9" />
        <line x1="1.3" y1="3" x2="8.7" y2="7" />
        <line x1="8.7" y1="3" x2="1.3" y2="7" />
      </svg>
    </span>
  {:else if prov === "gemini"}
    <span
      class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-blue-500 text-white shrink-0"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
        <path
          d="M5 0.5 L6.2 3.8 L9.5 5 L6.2 6.2 L5 9.5 L3.8 6.2 L0.5 5 L3.8 3.8 Z"
        />
      </svg>
    </span>
  {:else if prov === "grok"}
    <span
      class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-zinc-900 text-white shrink-0"
    >
      <svg
        width="10"
        height="10"
        viewBox="0 0 10 10"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
        fill="none"
      >
        <line x1="2" y1="2" x2="8" y2="8" />
        <line x1="8" y1="2" x2="2" y2="8" />
      </svg>
    </span>
  {:else if prov === "ollama"}
    <span
      class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-emerald-500 text-white shrink-0"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
        <circle cx="5" cy="3.2" r="1.8" />
        <path d="M2.5 9 C2.5 6.5 3.5 5.5 5 5.5 C6.5 5.5 7.5 6.5 7.5 9 Z" />
      </svg>
    </span>
  {:else}
    <span
      class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-slate-400 text-white shrink-0"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
        <circle cx="5" cy="5" r="3.5" />
      </svg>
    </span>
  {/if}
{/snippet}

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
    {@render providerIcon(provider)}
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
    <div
      class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400"
    >
      {PROVIDER_LABELS[provider] ?? provider} · Model
    </div>

    {#each models as m}
      <button
        type="button"
        class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-50 text-left transition-colors group"
        onclick={() => select(m)}
      >
        <div class="w-5 h-5 flex items-center justify-center shrink-0">
          {#if value === m}
            <Check size={13} class="text-primary" />
          {/if}
        </div>
        {@render providerIcon(provider)}
        <span
          class="font-mono text-[12px] flex-1 truncate {value === m
            ? 'text-primary font-semibold'
            : 'text-slate-700 group-hover:text-primary'} transition-colors"
        >
          {m}
        </span>
      </button>
    {/each}

    <!-- Custom model not in preset list -->
    {#if value && !models.includes(value)}
      <div class="mx-3 my-1 border-t border-slate-100"></div>
      <button
        type="button"
        class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-50 text-left transition-colors"
        onclick={() => select(value)}
      >
        <div class="w-5 h-5 flex items-center justify-center shrink-0">
          <Check size={13} class="text-primary" />
        </div>
        {@render providerIcon(provider)}
        <span
          class="font-mono text-[12px] text-primary font-semibold truncate flex-1"
        >
          {value}
          <span class="text-[10px] font-normal text-slate-400 ml-1"
            >(custom)</span
          >
        </span>
      </button>
    {/if}
  </div>
{/if}
