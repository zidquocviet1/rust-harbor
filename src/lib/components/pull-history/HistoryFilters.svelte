<script lang="ts">
  import {
    dateFrom,
    dateTo,
    groupByRepo,
    historyRepos,
    selectedRepo,
  } from "$lib/stores/pullHistoryStore";
  import { Layers } from "lucide-svelte";
  import DateRangeFilter from "./DateRangeFilter.svelte";
  import RepoFilterDropdown from "./RepoFilterDropdown.svelte";

  interface Props {
    totalCount: number;
    filteredCount: number;
  }

  let { totalCount, filteredCount }: Props = $props();

  const hasActiveFilters = $derived(
    !!$selectedRepo || !!$dateFrom || !!$dateTo,
  );
</script>

<div class="flex items-center gap-2.5 flex-wrap">
  <!-- Repository dropdown -->
  <RepoFilterDropdown
    repos={$historyRepos}
    value={$selectedRepo}
    onchange={(path) => selectedRepo.set(path)}
  />

  <!-- Date range filter (single element) -->
  <DateRangeFilter />

  <!-- Group by repo toggle -->
  <button
    type="button"
    onclick={() => groupByRepo.update((v) => !v)}
    class="flex items-center gap-1.5 bg-white/80 border rounded-xl px-3 py-2 text-sm font-medium cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/40 transition-all hover:bg-white {$groupByRepo
      ? 'border-primary/40 text-primary bg-primary/4 hover:border-primary/60'
      : 'border-slate-200/80 text-muted-foreground hover:border-slate-300/80'}"
    aria-pressed={$groupByRepo}
  >
    <Layers size={13} class="shrink-0" />
    Group
  </button>

  <!-- Clear all (only when both filters are active) -->
  {#if $selectedRepo && ($dateFrom || $dateTo)}
    <button
      type="button"
      class="text-sm font-semibold cursor-pointer transition-colors px-2 py-2 rounded-xl text-primary hover:bg-primary/8 border border-transparent hover:border-primary/20"
      onclick={() => {
        selectedRepo.set(null);
        dateFrom.set(null);
        dateTo.set(null);
      }}
    >
      Clear all
    </button>
  {/if}

  <!-- Count -->
  <span class="ml-auto text-sm font-medium text-muted-foreground">
    {#if filteredCount !== totalCount}
      Showing <span class="text-primary font-bold">{filteredCount}</span> of {totalCount}
    {:else}
      {totalCount} {totalCount === 1 ? "entry" : "entries"}
    {/if}
  </span>
</div>
