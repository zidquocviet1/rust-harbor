<script lang="ts">
  import type { Snippet } from "svelte";
  import { slide } from "svelte/transition";

  let {
    label,
    count,
    collapsed,
    onToggle,
    children
  }: {
    label: string;
    count: number;
    collapsed: boolean;
    onToggle: () => void;
    children: Snippet;
  } = $props();
</script>

<section class="relative space-y-3 rounded-[1.75rem] border border-slate-200/75 bg-slate-50/70 px-3 pb-3 pt-6">
  <header
    class="absolute -top-3 left-3 right-3 z-20 flex items-center justify-between cursor-pointer"
    onclick={onToggle}
    role="button"
    tabindex="0"
    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onToggle()}
  >
    <div class="flex-1 flex items-center justify-start">
      <div class="inline-flex items-center gap-2 rounded-full bg-slate-50/90 px-4 py-1.5 border border-slate-200/80 shadow-[0_6px_16px_rgba(15,23,42,0.08)]">
        <span class="text-[11px] font-black uppercase tracking-[0.2em]">{label}</span>
        <span class="text-[9px] uppercase tracking-[0.3em] text-muted-foreground">{count} repos</span>
      </div>
    </div>
    <div class="w-10 flex items-center justify-end">
      <div class="w-7 h-7 rounded-full bg-white/90 border border-slate-200/80 flex items-center justify-center text-muted-foreground transition-all">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="w-3.5 h-3.5 transform transition-transform duration-300 {collapsed ? '-rotate-90' : 'rotate-0'}"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </div>
    </div>
  </header>

  {#if !collapsed}
    <div class="overflow-hidden pt-2" transition:slide={{ duration: 220 }}>
      {@render children()}
    </div>
  {/if}
</section>
