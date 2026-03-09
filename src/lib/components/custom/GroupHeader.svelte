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

<section class="space-y-3">
  <header
    class="sticky top-0 z-20 -mx-2 px-4 py-2 rounded-2xl bg-background/80 backdrop-blur border border-white/10 flex items-center justify-between cursor-pointer group"
    onclick={onToggle}
    role="button"
    tabindex="0"
    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onToggle()}
  >
    <div class="flex items-center gap-3">
      <div class="w-6 h-6 rounded-xl bg-primary/10 border border-primary/30 flex items-center justify-center text-primary shadow-sm shadow-primary/30">
        <span class="text-[11px] font-black uppercase tracking-[0.14em]">
          {label.slice(0, 2)}
        </span>
      </div>
      <div>
        <p class="text-xs font-bold tracking-tight">{label}</p>
        <p class="text-[9px] text-muted-foreground uppercase tracking-[0.3em]">
          {count} repos
        </p>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <div class="h-6 px-2 rounded-full bg-white/5 border border-white/10 text-[9px] uppercase tracking-[0.3em] text-muted-foreground flex items-center gap-1">
        <span>{collapsed ? "Expand" : "Collapse"}</span>
      </div>
      <div class="w-7 h-7 rounded-full bg-white/5 border border-white/10 flex items-center justify-center text-muted-foreground group-hover:bg-primary/10 group-hover:text-primary transition-all">
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
