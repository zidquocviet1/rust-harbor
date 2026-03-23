<script lang="ts">
  import { Check, ChevronDown, GitBranch } from "lucide-svelte";

  interface Repo {
    path: string;
    name: string;
  }

  interface Props {
    repos: Repo[];
    value: string | null;
    onchange: (path: string | null) => void;
  }

  let { repos, value, onchange }: Props = $props();

  let isOpen = $state(false);
  let buttonEl = $state<HTMLButtonElement | null>(null);
  let dropdownStyle = $state("");

  const selectedName = $derived(
    value
      ? (repos.find((r) => r.path === value)?.name ?? value)
      : "All Repositories",
  );

  function toggle(e: MouseEvent) {
    e.stopPropagation();
    if (isOpen) {
      isOpen = false;
      return;
    }

    if (buttonEl) {
      const rect = buttonEl.getBoundingClientRect();
      const dropdownWidth = 220;
      let left = rect.left;
      if (left + dropdownWidth > window.innerWidth - 8)
        left = window.innerWidth - dropdownWidth - 8;
      dropdownStyle = `position: fixed; top: ${rect.bottom + 4}px; left: ${left}px; width: ${dropdownWidth}px;`;
    }
    isOpen = true;
  }

  function select(path: string | null) {
    onchange(path);
    isOpen = false;
  }

  function handleClickOutside(e: MouseEvent) {
    const t = e.target as HTMLElement;
    if (
      !t.closest(".repo-filter-trigger") &&
      !t.closest(".repo-filter-dropdown")
    ) {
      isOpen = false;
    }
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
    if (isOpen) {
      setTimeout(
        () => document.addEventListener("click", handleClickOutside),
        10,
      );
    }
    return () => document.removeEventListener("click", handleClickOutside);
  });
</script>

<div class="relative repo-filter-trigger">
  <button
    bind:this={buttonEl}
    type="button"
    onclick={toggle}
    class="flex items-center gap-2 bg-white/80 border border-slate-200/80 rounded-xl pl-3 pr-2.5 py-2 text-sm font-medium cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/40 focus:bg-white transition-all hover:bg-white hover:border-slate-300/80 {value
      ? 'text-foreground'
      : 'text-muted-foreground'}"
    style="min-width: 160px;"
  >
    <GitBranch
      size={13}
      class="shrink-0 {value ? 'text-primary' : 'text-muted-foreground'}"
    />
    <span class="flex-1 text-left truncate">{selectedName}</span>
    <ChevronDown
      size={12}
      class="shrink-0 text-muted-foreground transition-transform duration-150 {isOpen
        ? 'rotate-180'
        : ''}"
    />
  </button>
</div>

{#if isOpen}
  <div
    use:portal
    class="repo-filter-dropdown bg-white border border-slate-200 rounded-xl shadow-2xl py-2 z-[99999]"
    style={dropdownStyle}
  >
    <div
      class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400"
    >
      Repository
    </div>

    <!-- All option -->
    <button
      type="button"
      class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-100 text-left transition-colors group"
      onclick={() => select(null)}
    >
      <div class="w-5 h-5 flex items-center justify-center shrink-0">
        {#if value === null}
          <Check size={13} class="text-primary" />
        {/if}
      </div>
      <span
        class="text-sm font-medium {value === null
          ? 'text-primary'
          : 'text-slate-700 group-hover:text-primary'} transition-colors"
      >
        All Repositories
      </span>
    </button>

    {#if repos.length > 0}
      <div class="mx-3 my-1 border-t border-slate-100"></div>
      {#each repos as repo (repo.path)}
        <button
          type="button"
          class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-100 text-left transition-colors group"
          onclick={() => select(repo.path)}
        >
          <div class="w-5 h-5 flex items-center justify-center shrink-0">
            {#if value === repo.path}
              <Check size={13} class="text-primary" />
            {/if}
          </div>
          <span
            class="text-sm font-medium {value === repo.path
              ? 'text-primary'
              : 'text-slate-700 group-hover:text-primary'} transition-colors truncate"
          >
            {repo.name}
          </span>
        </button>
      {/each}
    {/if}
  </div>
{/if}
