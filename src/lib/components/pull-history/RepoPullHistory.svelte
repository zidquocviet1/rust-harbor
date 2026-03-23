<script lang="ts">
  import type { PullHistoryEntry } from "$lib/stores/pullHistoryStore";
  import { invoke } from "@tauri-apps/api/core";
  import { GitPullRequest, Loader2 } from "lucide-svelte";
  import PullHistoryCard from "./PullHistoryCard.svelte";

  interface Props {
    repoPath: string;
  }

  let { repoPath }: Props = $props();

  let entries = $state<PullHistoryEntry[]>([]);
  let loading = $state(true);

  $effect(() => {
    const path = repoPath;
    loading = true;
    entries = [];
    invoke<PullHistoryEntry[]>("list_pull_history", { repoPath: path })
      .then((data) => {
        entries = data;
      })
      .catch(() => {})
      .finally(() => {
        loading = false;
      });
  });

  function handleDeleted(id: number) {
    entries = entries.filter((e) => e.id !== id);
  }
</script>

{#if loading}
  <div
    class="flex items-center justify-center py-16 gap-3 text-muted-foreground"
  >
    <Loader2 size={15} class="animate-spin" />
    <span class="text-[13px]">Loading history…</span>
  </div>
{:else if entries.length === 0}
  <div
    class="flex flex-col items-center justify-center py-16 gap-3 text-muted-foreground"
  >
    <div
      class="w-12 h-12 rounded-2xl bg-muted flex items-center justify-center"
    >
      <GitPullRequest size={22} strokeWidth={1.8} class="opacity-40" />
    </div>
    <p class="text-[13px] font-medium">No pull history for this repository</p>
    <p class="text-[12px] text-muted-foreground/70">
      Pull from remote to start recording history.
    </p>
  </div>
{:else}
  <div class="flex flex-col gap-2">
    {#each entries as entry (entry.id)}
      <PullHistoryCard
        {entry}
        selectMode={false}
        selected={false}
        onToggleSelect={() => {}}
        onDeleted={() => handleDeleted(entry.id)}
      />
    {/each}
  </div>
{/if}
