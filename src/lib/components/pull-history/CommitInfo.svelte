<script lang="ts">
  import type { PullHistoryEntry } from "$lib/stores/pullHistoryStore";
  import { ArrowRight } from "lucide-svelte";

  interface Props {
    entry: PullHistoryEntry;
  }

  let { entry }: Props = $props();

  function formatDate(ts: number | null): string {
    if (!ts) return "";
    return new Date(ts * 1000).toLocaleString();
  }
</script>

<div class="grid grid-cols-[1fr_auto_1fr] items-start gap-2">
  <!-- Before -->
  <div class="rounded-2xl border border-slate-200/70 p-3.5 flex flex-col gap-1.5 bg-white/80">
    <div class="text-[11px] font-black uppercase tracking-widest text-muted-foreground flex items-center gap-1.5">
      <span class="w-1.5 h-1.5 rounded-full bg-border inline-block"></span>
      Before
    </div>

    <div class="font-mono text-[12px] font-semibold text-foreground break-all select-all leading-relaxed">
      {entry.commit_before}
    </div>

    {#if entry.commit_before_message}
      <div class="text-[13px] font-medium text-foreground leading-snug line-clamp-2" title={entry.commit_before_message}>
        {entry.commit_before_message}
      </div>
    {/if}

    {#if entry.commit_before_author || entry.commit_before_date}
      <div class="flex items-center gap-1 flex-wrap text-[12px] text-muted-foreground">
        {#if entry.commit_before_author}
          <span class="font-medium">{entry.commit_before_author}</span>
        {/if}
        {#if entry.commit_before_author && entry.commit_before_date}
          <span class="w-1 h-1 rounded-full bg-border shrink-0 inline-block"></span>
        {/if}
        {#if entry.commit_before_date}
          <span>{formatDate(entry.commit_before_date)}</span>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Arrow -->
  <div class="flex items-center justify-center w-7 h-7 rounded-full bg-primary/10 text-primary shrink-0 mt-7">
    <ArrowRight size={13} strokeWidth={2.5} />
  </div>

  <!-- After -->
  <div class="rounded-2xl border border-primary/25 p-3.5 flex flex-col gap-1.5 bg-primary/5">
    <div class="text-[11px] font-black uppercase tracking-widest text-primary flex items-center gap-1.5">
      <span class="w-1.5 h-1.5 rounded-full bg-primary inline-block"></span>
      After
    </div>

    <div class="font-mono text-[12px] font-semibold text-primary break-all select-all leading-relaxed">
      {entry.commit_after}
    </div>

    {#if entry.commit_after_message}
      <div class="text-[13px] font-medium text-foreground leading-snug line-clamp-2" title={entry.commit_after_message}>
        {entry.commit_after_message}
      </div>
    {/if}

    <div class="flex items-center gap-1 flex-wrap text-[12px] text-muted-foreground">
      {#if entry.commit_after_author}
        <span class="font-medium">{entry.commit_after_author}</span>
      {/if}
      {#if entry.commit_after_author && (entry.commit_after_date || entry.pulled_at)}
        <span class="w-1 h-1 rounded-full bg-border shrink-0 inline-block"></span>
      {/if}
      {#if entry.commit_after_date}
        <span>{formatDate(entry.commit_after_date)}</span>
      {:else}
        <span>Pulled {formatDate(entry.pulled_at)}</span>
      {/if}
    </div>
  </div>
</div>
