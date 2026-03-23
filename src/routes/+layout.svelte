<script lang="ts">
  import { page } from "$app/state";
  import GitMissingError from "$lib/components/custom/GitMissingError.svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import { APP_BRANDING } from "$lib/config/branding";
  import { unreadCount } from "$lib/stores/pullHistoryStore";
  import {
    allTags,
    createTag,
    deleteTag,
    loadTags,
    renameTag,
    selectedTagIds,
    toggleTagFilter,
    type Tag,
  } from "$lib/stores/tagStore";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, type Snippet } from "svelte";
  import { toast } from "svelte-sonner";
  import { fade, fly } from "svelte/transition";
  import "../app.css";

  let { children }: { children: Snippet } = $props();
  let gitInstalled = $state(true);
  let currentGitPath = $state("git");
  let checking = $state(true);
  let tagsCollapsed = $state(false);
  let creatingTag = $state(false);
  let newTagName = $state("");
  let newTagColor = $state("#6366f1");
  let renamingTagId = $state<number | null>(null);
  let renameValue = $state("");
  let contextMenuTag: Tag | null = $state(null);
  let contextMenuPosition = $state<{ x: number; y: number } | null>(null);
  let pendingDeleteTag: Tag | null = $state(null);
  let showDeleteDialog = $state(false);
  let tagSearch = $state("");

  async function checkGit() {
    try {
      const config = await invoke<any>("get_config");
      currentGitPath = config.git_path;
      gitInstalled = await invoke("is_git_installed");
    } catch (e) {
      console.error("Failed to check git:", e);
      gitInstalled = false;
    } finally {
      checking = false;
    }
  }

  onMount(async () => {
    await checkGit();
    await loadTags();

    const unlisten = await listen("config-changed", () => {
      checkGit();
    });

    return () => {
      unlisten();
    };
  });

  const PALETTE = [
    "#6366f1",
    "#22c55e",
    "#eab308",
    "#f97316",
    "#ec4899",
    "#06b6d4",
    "#3b82f6",
    "#a855f7",
    "#facc15",
    "#fb7185",
    "#4ade80",
    "#38bdf8",
  ];

  function handleCreateTag() {
    creatingTag = true;
    newTagName = "";
    newTagColor = PALETTE[0];
  }

  async function confirmCreateTag() {
    const name = newTagName.trim();
    if (!name) {
      toast.error("Tag name is required");
      return;
    }
    try {
      await createTag(name, newTagColor);
      creatingTag = false;
      newTagName = "";
    } catch (e: any) {
      const message =
        typeof e === "string" ? e : e?.message || "Failed to create tag";
      toast.error(message);
    }
  }

  function openRename(tag: Tag) {
    renamingTagId = tag.id;
    renameValue = tag.name;
  }

  async function confirmRename(tag: Tag) {
    const value = renameValue.trim();
    if (!value) {
      toast.error("Tag name is required");
      return;
    }
    try {
      await renameTag(tag.id, value);
      renamingTagId = null;
      renameValue = "";
    } catch (e: any) {
      const message =
        typeof e === "string" ? e : e?.message || "Failed to rename tag";
      toast.error(message);
    }
  }

  async function performDelete(tag: Tag) {
    try {
      await deleteTag(tag.id);
      toast.success("Tag deleted");
    } catch (e: any) {
      const message =
        typeof e === "string" ? e : e?.message || "Failed to delete tag";
      toast.error(message);
    } finally {
      pendingDeleteTag = null;
      showDeleteDialog = false;
    }
  }

  function handleTagContextMenu(event: MouseEvent, tag: Tag) {
    event.preventDefault();
    contextMenuTag = tag;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
  }

  function closeContextMenu() {
    contextMenuTag = null;
    contextMenuPosition = null;
  }
</script>

<svelte:head>
  <title>{APP_BRANDING.title}</title>
</svelte:head>

<Toaster position="bottom-right" richColors theme="light" />

<div class="fixed inset-0 bg-background -z-20"></div>
<div
  class="fixed inset-0 bg-gradient-to-tr from-primary/10 via-transparent to-sky-100/40 -z-10 pointer-events-none"
></div>

<!-- Decorative ambient glows -->
<div
  class="fixed top-[-10%] left-[-10%] w-[40%] h-[40%] bg-primary/12 rounded-full blur-[120px] -z-10 animate-pulse transition-all duration-1000"
></div>
<div
  class="fixed bottom-[-10%] right-[-10%] w-[30%] h-[30%] bg-cyan-400/10 rounded-full blur-[100px] -z-10 animate-pulse delay-700"
></div>

<main class="relative z-0">
  {#if checking}
    <div
      class="fixed inset-0 flex items-center justify-center bg-background/80 backdrop-blur-md z-[100]"
    >
      <div class="flex flex-col items-center space-y-6">
        <div class="relative">
          <div class="w-16 h-16 border-2 border-primary/20 rounded-full"></div>
          <div
            class="absolute inset-0 w-16 h-16 border-2 border-primary border-t-transparent rounded-full animate-spin"
          ></div>
          <div
            class="absolute inset-2 w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center"
          >
            <div class="w-1 h-1 bg-primary rounded-full animate-ping"></div>
          </div>
        </div>
        <div class="text-center space-y-1">
          <p class="text-xl font-bold tracking-tight text-glow">
            {APP_BRANDING.name}
          </p>
          <p
            class="text-xs text-muted-foreground uppercase tracking-[0.2em] font-medium animate-pulse"
          >
            Initializing System
          </p>
        </div>
      </div>
    </div>
  {:else if !gitInstalled}
    <GitMissingError currentPath={currentGitPath} />
  {:else}
    <div class="flex min-h-screen">
      <!-- Sidebar -->
      <aside
        class="w-72 border-r border-slate-200/70 glass flex flex-col sticky top-0 h-screen z-50 sidebar-shell"
      >
        <div
          class="p-5 flex items-center space-x-3 shadow-[0_1px_0_rgba(15,23,42,0.06)]"
        >
          <div
            class="p-2.5 bg-primary rounded-lg text-primary-foreground shadow-lg shadow-primary/20"
          >
            <svelte:component this={APP_BRANDING.icon} className="w-5 h-5" />
          </div>
          <div>
            <h1 class="sidebar-title font-semibold tracking-tight text-glow">
              {APP_BRANDING.shortName}
            </h1>
            <p class="sidebar-kicker font-semibold text-primary/80">
              {APP_BRANDING.tagline}
            </p>
          </div>
        </div>

        <nav class="flex-1 p-4 space-y-2 overflow-y-auto">
          <a
            href="/"
            class="sidebar-item flex items-center space-x-3 px-4 py-3 rounded-2xl transition-all duration-200 group {page
              .url.pathname === '/'
              ? 'bg-primary/12 text-primary border border-primary/30 shadow-sm shadow-primary/20'
              : 'hover:bg-slate-100/80 text-muted-foreground border border-transparent hover:border-slate-200/80'}"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="lucide lucide-layout-list group-hover:text-primary transition-colors"
              ><rect width="7" height="7" x="3" y="3" rx="1" /><rect
                width="7"
                height="7"
                x="14"
                y="3"
                rx="1"
              /><rect width="7" height="7" x="14" y="14" rx="1" /><rect
                width="7"
                height="7"
                x="3"
                y="14"
                rx="1"
              /></svg
            >
            <span class="font-medium">Repository List</span>
          </a>

          <!-- Pull History nav item (9.1 + 9.2) -->
          <a
            href="/pull-history"
            class="sidebar-item flex items-center space-x-3 px-4 py-3 rounded-2xl transition-all duration-200 group {page
              .url.pathname === '/pull-history'
              ? 'bg-primary/12 text-primary border border-primary/30 shadow-sm shadow-primary/20'
              : 'hover:bg-slate-100/80 text-muted-foreground border border-transparent hover:border-slate-200/80'}"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="group-hover:text-primary transition-colors"
              ><circle cx="18" cy="18" r="3" /><circle
                cx="6"
                cy="6"
                r="3"
              /><path d="M13 6h3a2 2 0 0 1 2 2v7" /><path
                d="M11 18H8a2 2 0 0 1-2-2V9"
              /><polyline points="11 15 8 18 11 21" /></svg
            >
            <span class="font-medium flex-1">Pull History</span>
            {#if $unreadCount > 0}
              <span
                class="inline-flex items-center justify-center min-w-[18px] h-[18px] px-1.5 rounded-full text-[10px] font-bold text-white"
                style="background: var(--color-primary);">{$unreadCount}</span
              >
            {/if}
          </a>

          <a
            href="/settings"
            class="sidebar-item flex items-center space-x-3 px-4 py-3 rounded-2xl transition-all duration-200 group {page
              .url.pathname === '/settings'
              ? 'bg-primary/12 text-primary border border-primary/30 shadow-sm shadow-primary/20'
              : 'hover:bg-slate-100/80 text-muted-foreground border border-transparent hover:border-slate-200/80'}"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="lucide lucide-folder-cog group-hover:text-primary transition-colors"
              ><path
                d="M10.5 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v3.3"
              /><circle cx="18" cy="18" r="3" /><path d="M18 14v1" /><path
                d="M18 21v1"
              /><path d="M22 18h-1" /><path d="M15 18h-1" /><path
                d="M21 15l-.7.7"
              /><path d="M15.7 20.3l-.7.7" /><path d="M21 21l-.7-.7" /><path
                d="M15.7 15.7l-.7-.7"
              /></svg
            >
            <span class="font-medium">Settings</span>
          </a>

          <!-- Tags Section -->
          <div class="mt-6 space-y-3">
            <div class="flex items-center justify-between px-4">
              <button
                type="button"
                class="sidebar-item flex items-center space-x-3 text-left group py-3"
                aria-expanded={!tagsCollapsed}
                onclick={() => (tagsCollapsed = !tagsCollapsed)}
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="w-[18px] h-[18px] text-muted-foreground group-hover:text-primary transition-colors"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><path
                    d="M20.59 13.41 11 3H4a1 1 0 0 0-1 1v7l9.59 9.59a2 2 0 0 0 2.82 0l5.18-5.18a2 2 0 0 0 0-2.82Z"
                  /><line x1="7" y1="7" x2="7.01" y2="7" /></svg
                >
                <span class="sidebar-item font-medium text-foreground"
                  >Tags</span
                >
              </button>
              <div class="flex items-center gap-1">
                <button
                  type="button"
                  aria-label="Create tag"
                  class="p-1.5 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary transition-colors"
                  onclick={(e) => {
                    e.stopPropagation();
                    handleCreateTag();
                  }}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="w-3.5 h-3.5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path d="M5 12h14" /><path d="M12 5v14" /></svg
                  >
                </button>
                <button
                  type="button"
                  aria-label={tagsCollapsed
                    ? "Expand tag section"
                    : "Collapse tag section"}
                  class="p-1.5 rounded-lg hover:bg-slate-100 text-muted-foreground hover:text-foreground transition-colors"
                  onclick={() => (tagsCollapsed = !tagsCollapsed)}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="w-3.5 h-3.5 transform transition-transform duration-300 {tagsCollapsed
                      ? '-rotate-90'
                      : 'rotate-0'}"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><polyline points="6 9 12 15 18 9" /></svg
                  >
                </button>
              </div>
            </div>

            {#if !tagsCollapsed}
              <div class="space-y-2 pl-8">
                <!-- Empty state -->
                {#if !$allTags.length && !creatingTag}
                  <button
                    type="button"
                    class="w-full px-3 py-3 rounded-xl border border-dashed border-slate-300/80 bg-white/80 text-[13px] text-muted-foreground flex items-center justify-between hover:border-slate-400 hover:bg-white transition-colors"
                    onclick={handleCreateTag}
                  >
                    <div class="flex items-center gap-2">
                      <div
                        class="w-7 h-7 rounded-full bg-primary/10 flex items-center justify-center"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          class="w-3.5 h-3.5 text-primary"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><path d="M12 5v14" /><path d="M5 12h14" /></svg
                        >
                      </div>
                      <span>Create your first tag</span>
                    </div>
                  </button>
                {/if}

                <!-- Create inline -->
                {#if creatingTag}
                  <div
                    class="px-3 py-3 rounded-xl border border-primary/40 bg-primary/5 space-y-3 animate-in fade-in slide-in-from-top-1 duration-200"
                  >
                    <input
                      class="w-full bg-white border border-slate-300/80 rounded-lg px-3 py-2 text-[13px] focus:outline-none focus:ring-2 focus:ring-primary/30"
                      placeholder="Tag name..."
                      bind:value={newTagName}
                      onkeydown={(e) => e.key === "Enter" && confirmCreateTag()}
                    />
                    <div class="flex flex-wrap gap-1.5">
                      {#each PALETTE as color}
                        <button
                          type="button"
                          aria-label={`Select color ${color}`}
                          class="w-5 h-5 rounded-full border {newTagColor ===
                          color
                            ? 'border-slate-700 shadow-md'
                            : 'border-slate-200'}"
                          style={`background-color: ${color}`}
                          onclick={() => (newTagColor = color)}
                        ></button>
                      {/each}
                    </div>
                    <div class="flex justify-end gap-2">
                      <button
                        class="text-[11px] uppercase tracking-[0.2em] text-muted-foreground hover:text-foreground"
                        onclick={() => {
                          creatingTag = false;
                          newTagName = "";
                        }}
                      >
                        Cancel
                      </button>
                      <button
                        class="text-[11px] uppercase tracking-[0.2em] text-primary hover:text-primary/80"
                        onclick={confirmCreateTag}
                      >
                        Create
                      </button>
                    </div>
                  </div>
                {/if}

                <!-- Search + list -->
                {#if $allTags.length}
                  <input
                    class="w-full bg-white/80 border border-slate-200/80 rounded-lg px-3 py-2 text-[12px] focus:outline-none focus:ring-2 focus:ring-primary/30"
                    placeholder="Search tags..."
                    bind:value={tagSearch}
                  />

                  <div class="space-y-1.5 max-h-64 overflow-y-auto pr-1">
                    {#each $allTags.filter((t) => t.name
                        .toLowerCase()
                        .includes(tagSearch
                            .trim()
                            .toLowerCase())) as tag (tag.id)}
                      {@const isSelected = $selectedTagIds.has(tag.id)}
                      <button
                        class="w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-[13px] transition-all text-muted-foreground hover:bg-slate-100 {isSelected
                          ? 'text-foreground'
                          : ''}"
                        style={isSelected
                          ? `background-color: ${tag.color}22`
                          : ""}
                        onclick={() => toggleTagFilter(tag.id)}
                        oncontextmenu={(e) =>
                          handleTagContextMenu(e as MouseEvent, tag)}
                        in:fly={{ y: -8, duration: 180 }}
                        out:fade={{ duration: 120 }}
                      >
                        <div class="flex items-center gap-2 min-w-0">
                          <span
                            class="w-2.5 h-2.5 rounded-full border border-slate-300 shrink-0"
                            style={`background: ${tag.color}`}
                          ></span>
                          {#if renamingTagId === tag.id}
                            <input
                              class="bg-white border border-slate-300 rounded px-2 py-1 text-[12px] w-full"
                              bind:value={renameValue}
                              onclick={(e) => e.stopPropagation()}
                              onkeydown={(e) => {
                                if (e.key === "Enter") confirmRename(tag);
                                if (e.key === "Escape") {
                                  renamingTagId = null;
                                  renameValue = "";
                                }
                              }}
                            />
                          {:else}
                            <span class="truncate">{tag.name}</span>
                          {/if}
                        </div>
                        <span
                          class="text-[10px] px-2 py-0.5 rounded-full bg-slate-100 border border-slate-200 text-foreground/70"
                        >
                          {tag.repo_count}
                        </span>
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        </nav>

        <div class="p-4 border-t border-slate-200/70 space-y-2">
          <div
            class="px-4 py-3 bg-white/70 rounded-2xl border border-slate-200/80"
          >
            <p
              class="text-[10px] uppercase tracking-widest text-muted-foreground font-bold mb-1"
            >
              Status
            </p>
            <div class="flex items-center space-x-2">
              <div
                class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"
              ></div>
              <span class="text-xs font-medium">System Ready</span>
            </div>
          </div>
        </div>
      </aside>

      <!-- Content Area -->
      <div class="flex-1 max-h-screen overflow-y-auto relative">
        {@render children()}
      </div>
    </div>
  {/if}
</main>

{#if contextMenuTag && contextMenuPosition}
  <button
    type="button"
    aria-label="Close tag context menu"
    class="fixed inset-0 z-[90]"
    onclick={closeContextMenu}
  ></button>
  <div
    class="fixed z-[91] w-40 bg-white/95 border border-slate-200 rounded-xl shadow-xl py-1 text-xs"
    style={`top: ${contextMenuPosition.y}px; left: ${contextMenuPosition.x}px;`}
  >
    <button
      type="button"
      class="w-full px-3 py-1.5 text-left hover:bg-slate-100 flex items-center justify-between"
      onclick={() => {
        if (contextMenuTag) openRename(contextMenuTag);
        closeContextMenu();
      }}
    >
      <span>Rename</span>
    </button>
    <button
      type="button"
      class="w-full px-3 py-1.5 text-left hover:bg-destructive/10 text-destructive"
      onclick={() => {
        if (contextMenuTag) {
          pendingDeleteTag = contextMenuTag;
          showDeleteDialog = true;
          closeContextMenu();
        }
      }}
    >
      <span>Delete</span>
    </button>
  </div>
{/if}

{#if showDeleteDialog && pendingDeleteTag}
  <div
    class="fixed inset-0 bg-slate-900/25 backdrop-blur-sm z-[95] flex items-center justify-center"
  >
    <div
      class="bg-white/95 border border-slate-200 rounded-2xl shadow-2xl p-5 w-[320px] space-y-4"
    >
      <div class="space-y-1">
        <p class="text-sm font-semibold">Delete tag</p>
        <p class="text-xs text-muted-foreground">
          Are you sure you want to delete tag "<span class="font-mono"
            >{pendingDeleteTag.name}</span
          >"? This will remove it from all repositories.
        </p>
      </div>
      <div class="flex justify-end gap-2">
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg text-xs uppercase tracking-[0.2em] text-muted-foreground hover:bg-slate-100"
          onclick={() => {
            showDeleteDialog = false;
            pendingDeleteTag = null;
          }}
        >
          Cancel
        </button>
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg text-xs uppercase tracking-[0.2em] bg-destructive text-destructive-foreground hover:bg-destructive/90"
          onclick={() => pendingDeleteTag && performDelete(pendingDeleteTag)}
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}
