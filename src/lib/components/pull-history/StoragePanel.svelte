<script lang="ts">
  import {
    type StorageStats,
    loadStorageStats,
  } from "$lib/stores/pullHistoryStore";
  import { AlertTriangle, ChevronDown, HardDrive, Trash2 } from "lucide-svelte";
  import { onMount } from "svelte";

  interface Props {
    /** Called when the user wants to select a set of entry IDs via this panel. */
    onSelectEntries: (ids: number[]) => void;
  }

  let { onSelectEntries }: Props = $props();

  let stats = $state<StorageStats | null>(null);
  let loading = $state(true);
  let expanded = $state(false);

  // Show at most 10 entries in the list
  const MAX_VISIBLE = 10;

  onMount(async () => {
    stats = await loadStorageStats();
    loading = false;
  });

  /** Reload after deletions so counts stay fresh. */
  export async function refresh() {
    stats = await loadStorageStats();
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024)
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function formatRelative(ts: number): string {
    const diff = Date.now() / 1000 - ts;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(ts * 1000).toLocaleDateString();
  }

  /** Size badge colour tier */
  function sizeTier(bytes: number): "xl" | "lg" | "md" | "sm" {
    if (bytes >= 50 * 1024 * 1024) return "xl"; // ≥ 50 MB
    if (bytes >= 10 * 1024 * 1024) return "lg"; // ≥ 10 MB
    if (bytes >= 1 * 1024 * 1024) return "md"; // ≥  1 MB
    return "sm";
  }

  const TIER_CLS: Record<string, string> = {
    xl: "bg-red-50 text-red-700 border-red-200/70",
    lg: "bg-amber-50 text-amber-700 border-amber-200/70",
    md: "bg-yellow-50 text-yellow-700 border-yellow-200/60",
    sm: "bg-muted text-muted-foreground border-border",
  };

  /** Entries considered "large" — top tier or any entry ≥ 10 MB */
  const largeEntries = $derived(
    stats?.entries.filter((e) => e.size_bytes >= 10 * 1024 * 1024) ?? [],
  );

  const visibleEntries = $derived(
    stats ? stats.entries.slice(0, MAX_VISIBLE) : [],
  );

  const hasMore = $derived((stats?.entries.length ?? 0) > MAX_VISIBLE);

  function selectLarge() {
    onSelectEntries(largeEntries.map((e) => e.id));
  }

  function selectEntry(id: number) {
    onSelectEntries([id]);
  }
</script>

{#if !loading && stats && stats.total_bytes > 0}
  <div
    class="bg-white/80 rounded-2xl border border-slate-200/70 overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center gap-3 px-4 py-3">
      <!-- Left: icon + text (clickable to expand) -->
      <button
        type="button"
        onclick={() => (expanded = !expanded)}
        class="flex items-center gap-3 flex-1 min-w-0 cursor-pointer text-left"
      >
        <div
          class="w-8 h-8 rounded-xl bg-primary/10 flex items-center justify-center shrink-0"
        >
          <HardDrive size={15} strokeWidth={2} class="text-primary" />
        </div>

        <div class="flex-1 min-w-0">
          <div
            class="text-[11px] font-black uppercase tracking-widest text-muted-foreground mb-0.5"
          >
            Storage Usage
          </div>
          <div class="flex items-center gap-2 flex-wrap">
            <span class="text-[14px] font-bold text-foreground">
              {formatBytes(stats.total_bytes)}
            </span>
            <span class="text-[11px] text-muted-foreground">
              across {stats.entries.length} pull{stats.entries.length === 1
                ? ""
                : "s"}
            </span>
            {#if largeEntries.length > 0}
              <span
                class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-semibold bg-amber-50 text-amber-700 border border-amber-200/70"
              >
                <AlertTriangle size={9} strokeWidth={2.5} />
                {largeEntries.length} large
              </span>
            {/if}
          </div>
        </div>

        <ChevronDown
          size={14}
          strokeWidth={2.5}
          class="shrink-0 text-muted-foreground transition-transform duration-140 {expanded
            ? 'rotate-180'
            : ''}"
        />
      </button>

      <!-- Quick select large button — separate from the expand toggle -->
      {#if largeEntries.length > 0}
        <button
          type="button"
          onclick={selectLarge}
          class="shrink-0 flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[11px] font-semibold cursor-pointer border border-amber-300/60 text-amber-700 bg-amber-50 hover:bg-amber-100 transition-colors"
        >
          <Trash2 size={11} />
          Select Large ({largeEntries.length})
        </button>
      {/if}
    </div>

    <!-- Expanded entry list -->
    {#if expanded}
      <div class="border-t border-slate-100 divide-y divide-slate-100/80">
        {#each visibleEntries as entry (entry.id)}
          {@const tier = sizeTier(entry.size_bytes)}
          <div
            class="flex items-center gap-3 px-4 py-2.5 hover:bg-slate-50/60 transition-colors group"
          >
            <!-- Size badge -->
            <span
              class="shrink-0 text-[11px] font-bold px-2 py-0.5 rounded-full border min-w-[56px] text-center {TIER_CLS[
                tier
              ]}"
            >
              {formatBytes(entry.size_bytes)}
            </span>

            <!-- Repo + branch -->
            <div class="flex-1 min-w-0 flex items-center gap-1.5 flex-wrap">
              <span class="text-[12px] font-semibold text-foreground truncate">
                {entry.repo_name}
              </span>
              <span
                class="text-[10px] font-semibold px-1.5 py-0.5 rounded-full bg-primary/10 text-primary shrink-0"
              >
                {entry.branch}
              </span>
              <span class="text-[11px] text-muted-foreground shrink-0">
                · {entry.files_changed_count} file{entry.files_changed_count ===
                1
                  ? ""
                  : "s"}
                · {formatRelative(entry.pulled_at)}
              </span>
            </div>

            <!-- Select button -->
            <button
              type="button"
              onclick={() => selectEntry(entry.id)}
              class="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity text-[10px] font-semibold px-2 py-1 rounded-lg border cursor-pointer text-muted-foreground border-slate-200/80 bg-white hover:border-destructive/40 hover:text-destructive hover:bg-destructive/5 transition-colors"
            >
              Select
            </button>
          </div>
        {/each}

        {#if hasMore}
          <div class="px-4 py-2 text-[11px] text-muted-foreground text-center">
            {stats!.entries.length - MAX_VISIBLE} more entries not shown
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}
