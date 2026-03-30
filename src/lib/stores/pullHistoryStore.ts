import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";

// ── Types ─────────────────────────────────────────────────────────────────────

/** Lightweight file metadata — no diff_content. Used in list/detail view. */
export interface PullHistoryFile {
  id: number;
  pull_id: number;
  file_path: string;
  change_type: "added" | "modified" | "deleted" | "renamed" | "binary";
  additions: number;
  deletions: number;
}

export interface PullHistoryEntry {
  id: number;
  repo_path: string;
  repo_name: string;
  branch: string;
  pulled_at: number; // Unix timestamp (seconds)
  commit_before: string;
  commit_after: string;
  files_changed_count: number;
  commit_before_date: number | null;
  commit_after_date: number | null;
  commit_before_message: string | null;
  commit_before_author: string | null;
  commit_after_message: string | null;
  commit_after_author: string | null;
  ai_summary: string | null;
  ai_provider: string | null;
  ai_model: string | null;
}

export interface PullHistoryDetail {
  entry: PullHistoryEntry;
  files: PullHistoryFile[];
}

export interface AiConfigPublic {
  provider: string;
  model: string;
  ollama_base_url: string | null;
  has_api_key: boolean;
  auth_method: string;
}

export interface PullResult {
  output: string;
  history_id: number | null;
}

export const PROVIDER_MODELS: Record<string, string[]> = {
  claude: ["claude-haiku-4-5-20251001", "claude-sonnet-4-6", "claude-opus-4-6"],
  openai: ["gpt-4o-mini", "gpt-4o", "gpt-4-turbo"],
  gemini: ["gemini-2.0-flash", "gemini-1.5-pro", "gemini-1.5-flash"],
  grok: ["grok-3", "grok-3-mini", "grok-2-1212"],
  ollama: ["llama3.2", "mistral", "qwen2.5-coder"],
};

// ── Core Stores ───────────────────────────────────────────────────────────────

export const pullHistoryEntries = writable<PullHistoryEntry[]>([]);
export const pullHistoryLoading = writable(false);
export const pullHistoryError = writable<string | null>(null);

/** Count of new pull history entries created since user last visited /pull-history */
export const unreadCount = writable(0);

// ── Filter State ──────────────────────────────────────────────────────────────

export const selectedRepo = writable<string | null>(null);
export const dateFrom = writable<Date | null>(null);
export const dateTo = writable<Date | null>(null);
export const groupByRepo = writable(false);

// ── Derived: filtered list ────────────────────────────────────────────────────

export const filteredEntries = derived(
  [pullHistoryEntries, selectedRepo, dateFrom, dateTo],
  ([$entries, $repo, $from, $to]) => {
    return $entries.filter((e) => {
      if ($repo && e.repo_path !== $repo) return false;
      if ($from && e.pulled_at * 1000 < $from.getTime()) return false;
      if ($to && e.pulled_at * 1000 > $to.getTime()) return false;
      return true;
    });
  }
);

// ── Grouped view ──────────────────────────────────────────────────────────────

export interface RepoGroup {
  repoPath: string;
  repoName: string;
  entries: PullHistoryEntry[];
}

export const repoGroups = derived(filteredEntries, ($filtered) => {
  const map = new Map<string, RepoGroup>();
  for (const entry of $filtered) {
    if (!map.has(entry.repo_path)) {
      map.set(entry.repo_path, {
        repoPath: entry.repo_path,
        repoName: entry.repo_name,
        entries: [],
      });
    }
    map.get(entry.repo_path)!.entries.push(entry);
  }
  return Array.from(map.values()).sort((a, b) =>
    a.repoName.localeCompare(b.repoName),
  );
});

/** Unique list of repos that appear in the history (for filter dropdown). */
export const historyRepos = derived(pullHistoryEntries, ($entries) => {
  const seen = new Map<string, { path: string; name: string }>();
  for (const e of $entries) {
    if (!seen.has(e.repo_path)) {
      seen.set(e.repo_path, { path: e.repo_path, name: e.repo_name });
    }
  }
  return Array.from(seen.values()).sort((a, b) => a.name.localeCompare(b.name));
});

// ── Actions ───────────────────────────────────────────────────────────────────

export async function loadPullHistory(repoPath?: string) {
  pullHistoryLoading.set(true);
  pullHistoryError.set(null);
  try {
    const entries = await invoke<PullHistoryEntry[]>("list_pull_history", {
      repoPath: repoPath ?? null,
    });
    pullHistoryEntries.set(entries);
  } catch (e: any) {
    pullHistoryError.set(typeof e === "string" ? e : "Failed to load pull history");
  } finally {
    pullHistoryLoading.set(false);
  }
}

export async function loadPullHistoryDetail(
  pullId: number
): Promise<PullHistoryDetail | null> {
  try {
    return await invoke<PullHistoryDetail>("get_pull_detail", { pullId });
  } catch (e: any) {
    console.error("[pullHistoryStore] loadPullHistoryDetail error:", e);
    return null;
  }
}

export interface PullEntrySize {
  id: number;
  repo_name: string;
  branch: string;
  pulled_at: number;
  files_changed_count: number;
  size_bytes: number;
}

export interface StorageStats {
  total_bytes: number;
  entries: PullEntrySize[];
}

export async function loadStorageStats(): Promise<StorageStats | null> {
  try {
    return await invoke<StorageStats>("get_pull_history_storage_stats");
  } catch (e: any) {
    console.error("[pullHistoryStore] loadStorageStats error:", e);
    return null;
  }
}

/** Fetches the diff content for a single file lazily (called on expand). */
export async function loadFileDiff(fileId: number): Promise<string> {
  try {
    return await invoke<string>("get_file_diff", { fileId });
  } catch (e: any) {
    console.error("[pullHistoryStore] loadFileDiff error:", e);
    return "";
  }
}

export async function deletePullHistoryEntry(pullId: number) {
  try {
    await invoke("remove_pull_history_entry", { pullId });
    pullHistoryEntries.update((entries) => entries.filter((e) => e.id !== pullId));
  } catch (e: any) {
    console.error("[pullHistoryStore] deletePullHistoryEntry error:", e);
    throw e;
  }
}

export async function deleteMultipleEntries(pullIds: number[]) {
  try {
    await invoke("remove_pull_history_entries", { pullIds });
    const idSet = new Set(pullIds);
    pullHistoryEntries.update((entries) => entries.filter((e) => !idSet.has(e.id)));
  } catch (e: any) {
    console.error("[pullHistoryStore] deleteMultipleEntries error:", e);
    throw e;
  }
}

export async function clearPullHistory() {
  try {
    await invoke("clear_all_pull_history");
    pullHistoryEntries.set([]);
  } catch (e: any) {
    console.error("[pullHistoryStore] clearPullHistory error:", e);
    throw e;
  }
}

export async function fetchPullHistoryCount(): Promise<number> {
  try {
    return await invoke<number>("pull_history_count");
  } catch {
    return 0;
  }
}

/** Call after a successful pull that returned a history_id. */
export function notifyNewPull(historyId: number | null) {
  if (historyId !== null) {
    unreadCount.update((n) => n + 1);
  }
}

/** Call when navigating to /pull-history to reset the badge. */
export function resetUnreadCount() {
  unreadCount.set(0);
}

// ── AI Summary ────────────────────────────────────────────────────────────────

export async function fetchAiConfig(): Promise<AiConfigPublic> {
  return invoke<AiConfigPublic>("get_ai_config");
}

export interface AiConfigsPublic {
  active_provider: string;
  providers: Record<string, {
    model: string;
    api_key: string | null;
    has_api_key: boolean;
    ollama_base_url: string | null;
    auth_method: string;
  }>;
}

export async function fetchAiConfigs(): Promise<AiConfigsPublic> {
  return invoke<AiConfigsPublic>("get_ai_configs");
}

export async function generatePullSummary(
  pullId: number,
  forceRegenerate = false,
  selectedModel?: string,
  selectedProvider?: string,
): Promise<string> {
  return invoke<string>("generate_pull_summary", {
    pullId,
    forceRegenerate,
    selectedModel: selectedModel ?? null,
    selectedProvider: selectedProvider ?? null,
  });
}
