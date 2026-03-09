<script lang="ts">
  import '../app.css';
  import { onMount, type Snippet } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from "svelte-sonner";
  import GitMissingError from '$lib/components/custom/GitMissingError.svelte';
  import { Toaster } from "$lib/components/ui/sonner";
  import { ModeWatcher } from "mode-watcher";
  import { page } from '$app/state';
  import { 
    allTags, 
    selectedTagIds, 
    tagLoading, 
    loadTags, 
    toggleTagFilter, 
    clearTagFilters, 
    createTag, 
    renameTag, 
    deleteTag,
    type Tag
  } from '$lib/stores/tagStore';

  let { children }: { children: Snippet } = $props();
  let gitInstalled = $state(true);
  let checking = $state(true);
  let tagsCollapsed = $state(false);
  let creatingTag = $state(false);
  let newTagName = $state("");
  let newTagColor = $state("#6366f1");
  let renamingTagId = $state<number | null>(null);
  let renameValue = $state("");
  let contextMenuTag: Tag | null = $state(null);
  let contextMenuPosition = $state<{ x: number; y: number } | null>(null);

  onMount(async () => {
    try {
      gitInstalled = await invoke('is_git_installed');
      await loadTags();
    } catch (e) {
      console.error('Failed to check git:', e);
      gitInstalled = false;
    } finally {
      checking = false;
    }
  });

  const PALETTE = [
    "#6366f1", "#22c55e", "#eab308", "#f97316",
    "#ec4899", "#06b6d4", "#3b82f6", "#a855f7",
    "#facc15", "#fb7185", "#4ade80", "#38bdf8"
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
      const message = typeof e === "string" ? e : (e?.message || "Failed to create tag");
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
      const message = typeof e === "string" ? e : (e?.message || "Failed to rename tag");
      toast.error(message);
    }
  }

  async function confirmDelete(tag: Tag) {
    try {
      await deleteTag(tag.id);
      toast.success("Tag deleted");
    } catch (e: any) {
      const message = typeof e === "string" ? e : (e?.message || "Failed to delete tag");
      toast.error(message);
    } finally {
      contextMenuTag = null;
      contextMenuPosition = null;
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
  <title>Rust Harbor — Local Git Command Center</title>
</svelte:head>

<ModeWatcher />
<Toaster position="bottom-right" richColors theme="dark" />

<div class="fixed inset-0 bg-background bg-grid-white -z-20"></div>
<div class="fixed inset-0 bg-gradient-to-tr from-primary/5 via-transparent to-transparent -z-10 pointer-events-none"></div>

<!-- Decorative ambient glows -->
<div class="fixed top-[-10%] left-[-10%] w-[40%] h-[40%] bg-primary/10 rounded-full blur-[120px] -z-10 animate-pulse transition-all duration-1000"></div>
<div class="fixed bottom-[-10%] right-[-10%] w-[30%] h-[30%] bg-indigo-500/10 rounded-full blur-[100px] -z-10 animate-pulse delay-700"></div>

<main class="relative z-0">
  {#if checking}
    <div class="fixed inset-0 flex items-center justify-center bg-background/80 backdrop-blur-md z-[100]">
      <div class="flex flex-col items-center space-y-6">
        <div class="relative">
          <div class="w-16 h-16 border-2 border-primary/20 rounded-full"></div>
          <div class="absolute inset-0 w-16 h-16 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
          <div class="absolute inset-2 w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center">
            <div class="w-1 h-1 bg-primary rounded-full animate-ping"></div>
          </div>
        </div>
        <div class="text-center space-y-1">
          <p class="text-xl font-bold tracking-tight text-glow">Rust Harbor</p>
          <p class="text-xs text-muted-foreground uppercase tracking-[0.2em] font-medium animate-pulse">Initializing System</p>
        </div>
      </div>
    </div>
  {:else if !gitInstalled}
    <GitMissingError />
  {:else}
    <div class="flex min-h-screen">
      <!-- Sidebar -->
      <aside class="w-64 border-r border-white/5 glass flex flex-col sticky top-0 h-screen z-50">
        <div class="p-6 border-b border-white/5 flex items-center space-x-3">
          <div class="p-2 bg-primary rounded-lg text-primary-foreground shadow-lg shadow-primary/20">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-anchor"><circle cx="12" cy="5" r="3"/><path d="M12 22V8"/><path d="M5 12H2a10 10 0 0 0 20 0h-3"/></svg>
          </div>
          <div>
            <h1 class="text-xl font-bold tracking-tight text-glow">Harbor</h1>
            <p class="text-[9px] uppercase tracking-[0.2em] font-semibold text-primary/80">Command Center</p>
          </div>
        </div>

        <nav class="flex-1 p-4 space-y-2 overflow-y-auto">
          <a 
            href="/" 
            class="flex items-center space-x-3 px-4 py-3 rounded-xl transition-all duration-300 group {page.url.pathname === '/' ? 'bg-primary/20 text-primary' : 'hover:bg-white/5 text-muted-foreground'}"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-layout-list group-hover:text-primary transition-colors"><rect width="7" height="7" x="3" y="3" rx="1"/><rect width="7" height="7" x="14" y="3" rx="1"/><rect width="7" height="7" x="14" y="14" rx="1"/><rect width="7" height="7" x="3" y="14" rx="1"/></svg>
            <span class="font-medium">Repository List</span>
          </a>
          <a 
            href="/settings" 
            class="flex items-center space-x-3 px-4 py-3 rounded-xl transition-all duration-300 group {page.url.pathname === '/settings' ? 'bg-primary/20 text-primary' : 'hover:bg-white/5 text-muted-foreground'}"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-folder-cog group-hover:text-primary transition-colors"><path d="M10.5 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v3.3"/><circle cx="18" cy="18" r="3"/><path d="M18 14v1"/><path d="M18 21v1"/><path d="M22 18h-1"/><path d="M15 18h-1"/><path d="M21 15l-.7.7"/><path d="M15.7 20.3l-.7.7"/><path d="M21 21l-.7-.7"/><path d="M15.7 15.7l-.7-.7"/></svg>
            <span class="font-medium">Watched Folders</span>
          </a>

          <!-- Tags Section -->
          <div class="mt-6">
            <div 
              role="button"
              tabindex="0"
              class="flex items-center justify-between px-4 py-2 rounded-xl bg-white/5 border border-white/10 cursor-pointer group text-left"
              onclick={() => tagsCollapsed = !tagsCollapsed}
              onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (tagsCollapsed = !tagsCollapsed)}
            >
              <div class="flex items-center gap-2">
                <div class="p-1.5 rounded-lg bg-primary/10 text-primary shadow-sm shadow-primary/30">
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v2.5"/><path d="M3 7v12a2 2 0 0 0 2 2h9"/><path d="M18 15v6"/><path d="M15 18h6"/></svg>
                </div>
                <div class="flex flex-col">
                  <span class="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground/60">
                    Tags
                  </span>
                  <span class="text-xs font-medium text-muted-foreground">
                    {$allTags.reduce((acc, t) => acc + t.repo_count, 0)} repos
                  </span>
                </div>
              </div>
              <div class="flex items-center gap-1">
                <button 
                  type="button"
                  aria-label="Create tag"
                  class="p-1.5 rounded-lg hover:bg-primary/10 text-muted-foreground hover:text-primary transition-colors"
                  onclick={(e) => { e.stopPropagation(); handleCreateTag(); }}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                </button>
                <button 
                  type="button"
                  aria-label={tagsCollapsed ? "Expand tag section" : "Collapse tag section"}
                  class="p-1.5 rounded-lg hover:bg-white/10 text-muted-foreground hover:text-foreground transition-colors"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3 transform transition-transform duration-300 {tagsCollapsed ? '-rotate-90' : 'rotate-0'}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
                </button>
              </div>
            </div>

            {#if !tagsCollapsed}
              <div class="mt-3 space-y-2">
                <!-- Empty state -->
                {#if !$allTags.length && !creatingTag}
                  <div class="px-4 py-3 rounded-xl border border-dashed border-white/10 bg-white/5 text-xs text-muted-foreground flex items-center justify-between">
                    <div class="flex items-center gap-2">
                      <div class="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14"/><path d="M5 12h14"/></svg>
                      </div>
                      <span>Create your first tag</span>
                    </div>
                    <button 
                      class="text-[10px] font-bold uppercase tracking-[0.2em] text-primary hover:text-primary/80"
                      onclick={handleCreateTag}
                    >
                      New
                    </button>
                  </div>
                {/if}

                <!-- Create inline -->
                {#if creatingTag}
                  <div class="px-4 py-3 rounded-xl border border-primary/40 bg-primary/5 space-y-3 animate-in fade-in slide-in-from-top-1 duration-200">
                    <input
                      class="w-full bg-black/20 border border-white/10 rounded-lg px-3 py-1.5 text-xs focus:outline-none focus:ring-1 focus:ring-primary/60"
                      placeholder="Tag name..."
                      bind:value={newTagName}
                      onkeydown={(e) => e.key === 'Enter' && confirmCreateTag()}
                    />
                    <div class="flex flex-wrap gap-1">
                      {#each PALETTE as color}
                        <button
                          type="button"
                          aria-label={`Select color ${color}`}
                          class="w-5 h-5 rounded-full border {newTagColor === color ? 'border-white shadow-glow' : 'border-white/20'}"
                          style={`background-color: ${color}`}
                          onclick={() => newTagColor = color}
                        ></button>
                      {/each}
                    </div>
                    <div class="flex justify-end gap-2">
                      <button
                        class="text-[10px] uppercase tracking-[0.2em] text-muted-foreground hover:text-foreground"
                        onclick={() => { creatingTag = false; newTagName = ''; }}
                      >
                        Cancel
                      </button>
                      <button
                        class="text-[10px] uppercase tracking-[0.2em] text-primary hover:text-primary/80"
                        onclick={confirmCreateTag}
                      >
                        Create
                      </button>
                    </div>
                  </div>
                {/if}

                <!-- "All" + list -->
                {#if $allTags.length}
                  <div class="flex items-center justify-between px-4 text-[10px] text-muted-foreground/70 mb-1">
                    <button
                      class="uppercase tracking-[0.2em] font-bold hover:text-foreground { $selectedTagIds.size === 0 ? 'text-primary' : '' }"
                      onclick={clearTagFilters}
                    >
                      All
                    </button>
                    <span class="uppercase tracking-[0.2em]">
                      {#if $tagLoading}Loading...{:else}{$allTags.length} tags{/if}
                    </span>
                  </div>

                  <div class="space-y-1 max-h-64 overflow-y-auto pr-1">
                    {#each $allTags as tag}
                      {@const isSelected = $selectedTagIds.has(tag.id)}
                      <button
                        class="w-full flex items-center justify-between px-4 py-2 rounded-lg text-xs transition-all border {isSelected ? 'bg-primary/15 border-primary/50 text-primary shadow-glow' : 'bg-white/3 border-transparent hover:bg-white/8 text-muted-foreground'}"
                        onclick={() => toggleTagFilter(tag.id)}
                        oncontextmenu={(e) => handleTagContextMenu(e as MouseEvent, tag)}
                      >
                        <div class="flex items-center gap-2 min-w-0">
                          <span
                            class="w-2 h-2 rounded-full border border-white/20 shrink-0"
                            style={`background: ${tag.color}`}
                          ></span>
                          {#if renamingTagId === tag.id}
                            <input
                              class="bg-black/30 border border-white/10 rounded px-2 py-0.5 text-[11px] w-full"
                              bind:value={renameValue}
                              onclick={(e) => e.stopPropagation()}
                              onkeydown={(e) => {
                                if (e.key === 'Enter') confirmRename(tag);
                                if (e.key === 'Escape') { renamingTagId = null; renameValue = ''; }
                              }}
                            />
                          {:else}
                            <span class="truncate">{tag.name}</span>
                          {/if}
                        </div>
                        <span class="text-[9px] px-2 py-0.5 rounded-full bg-black/30 border border-white/10">
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

        <div class="p-4 border-t border-white/5 space-y-2">
          <div class="px-4 py-3 bg-white/5 rounded-2xl border border-white/5">
            <p class="text-[10px] uppercase tracking-widest text-muted-foreground font-bold mb-1">Status</p>
            <div class="flex items-center space-x-2">
              <div class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></div>
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

