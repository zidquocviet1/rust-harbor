<script lang="ts">
  import {
    type PullHistoryDetail,
    type PullHistoryEntry,
    deletePullHistoryEntry,
    loadPullHistoryDetail,
  } from "$lib/stores/pullHistoryStore";
  import {
    Check,
    ChevronDown,
    GitPullRequest,
    Loader2,
    Trash2,
  } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import CommitInfo from "./CommitInfo.svelte";
  import FileList from "./FileList.svelte";

  interface Props {
    entry: PullHistoryEntry;
    selectMode: boolean;
    selected: boolean;
    onToggleSelect: () => void;
    onDeleted: () => void;
  }

  let { entry, selectMode, selected, onToggleSelect, onDeleted }: Props =
    $props();

  let expanded = $state(false);
  let detail = $state<PullHistoryDetail | null>(null);
  let loadingDetail = $state(false);
  let showDeleteConfirm = $state(false);
  let deleting = $state(false);

  async function toggleExpand() {
    if (selectMode) {
      onToggleSelect();
      return;
    }
    expanded = !expanded;
    if (expanded && !detail) {
      loadingDetail = true;
      detail = await loadPullHistoryDetail(entry.id);
      loadingDetail = false;
    }
  }

  async function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    if (!showDeleteConfirm) {
      showDeleteConfirm = true;
      return;
    }
    deleting = true;
    try {
      await deletePullHistoryEntry(entry.id);
      onDeleted();
      toast.success("Pull history entry deleted");
    } catch {
      toast.error("Failed to delete entry");
    } finally {
      deleting = false;
      showDeleteConfirm = false;
    }
  }

  function cancelDelete(e: MouseEvent) {
    e.stopPropagation();
    showDeleteConfirm = false;
  }

  function formatRelativeTime(ts: number): string {
    const diff = Date.now() / 1000 - ts;
    if (diff < 60) return "Just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(ts * 1000).toLocaleDateString();
  }

  function formatAbsTime(ts: number): string {
    return new Date(ts * 1000).toLocaleString();
  }

  function shortSha(sha: string) {
    return sha.slice(0, 7);
  }

  const totalAdditions = $derived(
    detail?.files.reduce((s, f) => s + f.additions, 0) ?? 0,
  );
  const totalDeletions = $derived(
    detail?.files.reduce((s, f) => s + f.deletions, 0) ?? 0,
  );
</script>

<div
  class="group glass repo-card-enter rounded-[1.5rem] overflow-hidden
         transition-[background-color,border-color,box-shadow,transform] duration-200
         hover:bg-white hover:border-slate-300/90 hover:shadow-[0_10px_22px_rgba(15,23,42,0.09)] hover:-translate-y-0.5"
  class:border-primary={expanded}
  style="{expanded
    ? 'box-shadow: 0 0 0 1.5px oklch(0.57 0.19 258 / 0.15), 0 4px 24px oklch(0.57 0.19 258 / 0.08);'
    : ''} --stagger-index: 0;"
>
  <!-- ── Card Header ──────────────────────────────────────────────────────── -->
  <div
    role="button"
    tabindex="0"
    class="flex items-center gap-3.5 px-5 py-4 cursor-pointer select-none transition-colors"
    class:bg-primary={expanded}
    style={expanded ? "background: oklch(0.57 0.19 258 / 0.04);" : ""}
    onclick={toggleExpand}
    onkeydown={(e) => e.key === "Enter" && toggleExpand()}
    aria-expanded={expanded}
  >
    <!-- Checkbox (select mode) or icon -->
    {#if selectMode}
      <div
        class="w-[18px] h-[18px] rounded flex items-center justify-center shrink-0 transition-all border"
        class:bg-primary={selected}
        class:border-primary={selected}
        class:border-border={!selected}
      >
        {#if selected}
          <Check size={11} class="text-primary-foreground" strokeWidth={3.5} />
        {/if}
      </div>
    {:else}
      <div
        class="w-[42px] h-[42px] min-w-[42px] rounded-2xl flex items-center justify-center shrink-0 bg-primary/10 text-primary transition-all duration-200 group-hover:bg-primary/15"
      >
        <GitPullRequest size={17} strokeWidth={2.2} />
      </div>
    {/if}

    <!-- Main content -->
    <div class="flex-1 flex flex-col gap-1.5 min-w-0">
      <!-- Row 1: name + branch -->
      <div class="flex items-center gap-2 flex-wrap">
        <span
          class="text-[15px] font-black tracking-tight truncate text-foreground group-hover:text-primary transition-colors"
          >{entry.repo_name}</span
        >
        <span
          class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold shrink-0 bg-primary/10 text-primary"
        >
          {entry.branch}
        </span>
      </div>

      <!-- Row 2: SHA range + timestamp + files -->
      <div class="flex items-center gap-1.5 flex-wrap text-[12px]">
        <!-- SHA range -->
        <div class="flex items-center gap-1">
          <span
            class="font-mono text-[10.5px] px-1.5 py-0.5 rounded-[var(--radius-sm)] font-medium bg-muted text-muted-foreground cursor-default"
            title={entry.commit_before}
          >
            {shortSha(entry.commit_before)}
          </span>
          <span class="text-muted-foreground">→</span>
          <span
            class="font-mono text-[10.5px] px-1.5 py-0.5 rounded-[var(--radius-sm)] font-medium bg-primary/10 text-primary cursor-default"
            title={entry.commit_after}
          >
            {shortSha(entry.commit_after)}
          </span>
        </div>

        <span class="w-1 h-1 rounded-full shrink-0 bg-border"></span>

        <!-- Timestamp -->
        <span
          title={formatAbsTime(entry.pulled_at)}
          class="text-muted-foreground font-medium"
        >
          {formatRelativeTime(entry.pulled_at)}
        </span>

        <span class="w-1 h-1 rounded-full shrink-0 bg-border"></span>

        <!-- Files count -->
        <span
          class="flex items-center gap-1 font-semibold text-muted-foreground"
        >
          <svg
            width="11"
            height="11"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
          >
            <path
              d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
            />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          {entry.files_changed_count}
          {entry.files_changed_count === 1 ? "file" : "files"}
        </span>
      </div>
    </div>

    <!-- Actions -->
    {#if !selectMode}
      <div class="flex items-center gap-1.5 ml-auto shrink-0">
        {#if showDeleteConfirm}
          <span class="text-[11.5px] font-semibold mr-1 text-destructive"
            >Delete?</span
          >
          <button
            type="button"
            class="h-7 px-2 rounded-[var(--radius-sm)] text-[11px] font-semibold cursor-pointer border text-destructive border-destructive/35 bg-transparent hover:bg-destructive/10 transition-colors"
            onclick={handleDelete}
            disabled={deleting}>Yes</button
          >
          <button
            type="button"
            class="h-7 px-2 rounded-[var(--radius-sm)] text-[11px] font-semibold cursor-pointer border text-muted-foreground border-border bg-transparent hover:bg-muted transition-colors"
            onclick={cancelDelete}>No</button
          >
        {:else}
          <button
            type="button"
            aria-label="Delete entry"
            class="w-[30px] h-[30px] rounded-[var(--radius-sm)] flex items-center justify-center border border-transparent cursor-pointer transition-all duration-140 hover:border-destructive/30 hover:bg-destructive/8 text-muted-foreground hover:text-destructive"
            onclick={handleDelete}
          >
            <Trash2 size={13} />
          </button>
        {/if}

        <!-- Chevron -->
        <div
          class="w-[30px] h-[30px] flex items-center justify-center text-muted-foreground"
        >
          <ChevronDown
            size={14}
            strokeWidth={2.5}
            class="transition-transform duration-140"
            style="transform: {expanded ? 'rotate(180deg)' : 'rotate(0deg)'};"
          />
        </div>
      </div>
    {/if}
  </div>

  <!-- ── Detail Panel ─────────────────────────────────────────────────────── -->
  {#if expanded}
    <div
      class="border-t border-border p-4 flex flex-col gap-4"
      style="background: oklch(0.985 0.004 220 / 0.50);"
    >
      <CommitInfo {entry} />

      {#if loadingDetail}
        <div class="flex items-center gap-2 text-[13px] text-muted-foreground">
          <Loader2 size={14} class="animate-spin" />
          Loading changes…
        </div>
      {:else if detail && detail.files.length > 0}
        <FileList files={detail.files} {totalAdditions} {totalDeletions} />
      {:else if detail}
        <p class="text-[13px] text-muted-foreground">
          No file changes recorded for this pull.
        </p>
      {/if}
    </div>
  {/if}
</div>
