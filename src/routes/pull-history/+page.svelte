<script lang="ts">
  import EmptyState from "$lib/components/pull-history/EmptyState.svelte";
  import HistoryFilters from "$lib/components/pull-history/HistoryFilters.svelte";
  import PullHistoryCard from "$lib/components/pull-history/PullHistoryCard.svelte";
  import StoragePanel from "$lib/components/pull-history/StoragePanel.svelte";
  import {
    clearPullHistory,
    deleteMultipleEntries,
    filteredEntries,
    groupByRepo,
    loadPullHistory,
    pullHistoryEntries,
    pullHistoryLoading,
    repoGroups,
    resetUnreadCount,
  } from "$lib/stores/pullHistoryStore";
  import { CheckSquare, ChevronDown, GitBranch, Loader2, Trash2 } from "lucide-svelte";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  // ── State ────────────────────────────────────────────────────────────────────
  let selectMode = $state(false);
  let selectedIds = $state<Set<number>>(new Set());
  let showClearConfirm = $state(false);
  let clearing = $state(false);
  let deletingBulk = $state(false);

  // ── Lifecycle ────────────────────────────────────────────────────────────────
  onMount(async () => {
    await loadPullHistory();
    resetUnreadCount();
  });

  // ── Helpers ──────────────────────────────────────────────────────────────────
  function toggleSelect(id: number) {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedIds = next;
  }

  function exitSelectMode() {
    selectMode = false;
    selectedIds = new Set();
  }

  async function handleBulkDelete() {
    if (selectedIds.size === 0) return;
    deletingBulk = true;
    try {
      await deleteMultipleEntries([...selectedIds]);
      toast.success(`Deleted ${selectedIds.size} entries`);
      exitSelectMode();
      storagePanel?.refresh();
    } catch {
      toast.error("Failed to delete selected entries");
    } finally {
      deletingBulk = false;
    }
  }

  async function handleClearAll() {
    if (!showClearConfirm) {
      showClearConfirm = true;
      return;
    }
    clearing = true;
    try {
      await clearPullHistory();
      toast.success("All pull history cleared");
      storagePanel?.refresh();
    } catch {
      toast.error("Failed to clear history");
    } finally {
      clearing = false;
      showClearConfirm = false;
    }
  }

  function handleEntryDeleted(id: number) {
    pullHistoryEntries.update((e) => e.filter((x) => x.id !== id));
    storagePanel?.refresh();
  }

  const totalCount = $derived($pullHistoryEntries.length);
  const filteredCount = $derived($filteredEntries.length);

  // Collapsed state for repo groups
  let collapsedGroups = $state<Set<string>>(new Set());

  // Storage panel ref — refreshed after bulk deletes
  let storagePanel = $state<ReturnType<typeof StoragePanel> | null>(null);

  /** Activate select mode and pre-select the given IDs (from StoragePanel). */
  function handleSelectEntries(ids: number[]) {
    selectMode = true;
    selectedIds = new Set(ids);
  }

  function toggleGroup(repoPath: string) {
    const next = new Set(collapsedGroups);
    next.has(repoPath) ? next.delete(repoPath) : next.add(repoPath);
    collapsedGroups = next;
  }
</script>

<!-- ── Page ──────────────────────────────────────────────────────────────────── -->
<div
  class="p-8 space-y-8 animate-in fade-in duration-500 max-w-5xl mx-auto min-h-screen"
>
  <!-- ── Page Header ──────────────────────────────────────────────────────── -->
  <div class="flex items-start justify-between gap-4">
    <div class="space-y-1">
      <h1 class="text-5xl font-black tracking-tighter text-glow mb-2">
        Pull History
      </h1>
      <p class="text-muted-foreground text-xl">
        <span class="text-primary font-bold">{totalCount}</span>
        pull{totalCount === 1 ? "" : "s"} recorded
      </p>
    </div>

    <!-- Actions -->
    {#if totalCount > 0}
      <div class="flex items-center gap-2 pt-1">
        <!-- Select toggle -->
        <button
          type="button"
          class="flex items-center gap-1.5 px-3.5 py-2 rounded-xl border text-sm font-semibold cursor-pointer transition-all duration-150 {selectMode
            ? 'bg-primary text-primary-foreground border-primary'
            : 'bg-white/80 text-foreground border-slate-200/80 hover:bg-white'}"
          onclick={() => {
            selectMode = !selectMode;
            if (!selectMode) selectedIds = new Set();
          }}
        >
          <CheckSquare size={13} />
          {selectMode ? "Cancel" : "Select"}
        </button>

        <!-- Clear all / confirm -->
        {#if showClearConfirm}
          <span class="text-sm font-semibold text-destructive"
            >Clear all history?</span
          >
          <button
            type="button"
            class="px-3 py-2 rounded-xl border text-sm font-semibold cursor-pointer text-destructive border-destructive/35 bg-destructive/8 hover:bg-destructive/15 transition-colors"
            onclick={handleClearAll}
            disabled={clearing}>Yes, clear</button
          >
          <button
            type="button"
            class="px-3 py-2 rounded-xl border text-sm font-semibold cursor-pointer text-muted-foreground border-slate-200/80 bg-white/80 hover:bg-white transition-colors"
            onclick={() => (showClearConfirm = false)}>Cancel</button
          >
        {:else}
          <button
            type="button"
            class="flex items-center gap-1.5 px-3.5 py-2 rounded-xl border text-sm font-semibold cursor-pointer text-destructive border-destructive/35 bg-transparent hover:bg-destructive/8 transition-colors"
            onclick={handleClearAll}
          >
            <Trash2 size={13} />
            Clear All
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <!-- ── Filter Bar ────────────────────────────────────────────────────────── -->
  {#if totalCount > 0}
    <HistoryFilters {totalCount} {filteredCount} />
    <StoragePanel bind:this={storagePanel} onSelectEntries={handleSelectEntries} />
  {/if}

  <!-- ── Loading ──────────────────────────────────────────────────────────── -->
  {#if $pullHistoryLoading}
    <div
      class="flex items-center justify-center py-16 gap-3 text-muted-foreground"
    >
      <Loader2 size={16} class="animate-spin" />
      Loading history…
    </div>

    <!-- ── Empty state ──────────────────────────────────────────────────────── -->
  {:else if totalCount === 0}
    <div class="glass rounded-[var(--radius-lg)]">
      <EmptyState />
    </div>

    <!-- ── History list ─────────────────────────────────────────────────────── -->
  {:else}
    <div
      class="flex flex-col gap-2.5"
      style="padding-bottom: {selectMode && selectedIds.size > 0 ? '90px' : '0'};"
    >
      {#if $filteredEntries.length === 0}
        <div class="glass rounded-[var(--radius-lg)] py-10 text-center text-[13px] text-muted-foreground">
          No entries match the current filters.
        </div>
      {:else if $groupByRepo}
        <!-- ── Grouped by repository ── -->
        {#each $repoGroups as group (group.repoPath)}
          {@const collapsed = collapsedGroups.has(group.repoPath)}

          <button
            type="button"
            onclick={() => toggleGroup(group.repoPath)}
            class="w-full flex items-center gap-2.5 px-3 py-2.5 rounded-xl bg-white/80 border border-slate-200/80 hover:bg-white hover:border-slate-300/80 transition-all cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/40"
            aria-expanded={!collapsed}
          >
            <GitBranch size={14} class="text-primary shrink-0" />
            <span class="font-semibold text-sm text-foreground flex-1 text-left">
              {group.repoName}
            </span>
            <span class="text-[11px] font-medium text-muted-foreground bg-muted px-2 py-0.5 rounded-full border border-border shrink-0">
              {group.entries.length}{group.entries.length === 1 ? " pull" : " pulls"}
            </span>
            <ChevronDown
              size={13}
              class="text-muted-foreground shrink-0 transition-transform duration-140 {collapsed ? '-rotate-90' : ''}"
            />
          </button>

          {#if !collapsed}
            <div class="flex flex-col gap-2 ml-3 pl-3 border-l border-border/60">
              {#each group.entries as entry (entry.id)}
                <PullHistoryCard
                  {entry}
                  {selectMode}
                  selected={selectedIds.has(entry.id)}
                  onToggleSelect={() => toggleSelect(entry.id)}
                  onDeleted={() => handleEntryDeleted(entry.id)}
                />
              {/each}
            </div>
          {/if}
        {/each}
      {:else}
        <!-- ── Flat list ── -->
        {#each $filteredEntries as entry (entry.id)}
          <PullHistoryCard
            {entry}
            {selectMode}
            selected={selectedIds.has(entry.id)}
            onToggleSelect={() => toggleSelect(entry.id)}
            onDeleted={() => handleEntryDeleted(entry.id)}
          />
        {/each}
      {/if}
    </div>
  {/if}
</div>

<!-- ── Floating multi-select toolbar ────────────────────────────────────────── -->
{#if selectMode && selectedIds.size > 0}
  <div
    class="fixed bottom-6 left-1/2 -translate-x-1/2 flex items-center gap-3 px-5 py-3 rounded-[var(--radius-lg)] border z-50 glass"
    style="box-shadow: 0 8px 40px oklch(0 0 0 / 0.12);"
  >
    <div
      class="w-[22px] h-[22px] rounded-full flex items-center justify-center text-[11px] font-bold text-primary-foreground shrink-0 bg-primary"
    >
      {selectedIds.size}
    </div>
    <span class="text-[13px] font-semibold text-foreground">
      {selectedIds.size}
      {selectedIds.size === 1 ? "entry" : "entries"} selected
    </span>
    <button
      type="button"
      class="px-3 py-1.5 rounded-xl border text-sm font-semibold cursor-pointer text-muted-foreground border-slate-200/80 bg-white/80 hover:bg-white transition-colors"
      onclick={exitSelectMode}>Cancel</button
    >
    <button
      type="button"
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl border text-sm font-semibold cursor-pointer text-destructive border-destructive/35 bg-transparent hover:bg-destructive/8 transition-colors"
      onclick={handleBulkDelete}
      disabled={deletingBulk}
    >
      <Trash2 size={12} />
      Delete Selected ({selectedIds.size})
    </button>
  </div>
{/if}
