<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { toast } from "svelte-sonner";
  import Fuse from "fuse.js";
  import { 
    RefreshCw, 
    GitBranch, 
    FolderOpen, 
    ExternalLink,
    ArrowDown,
    ArrowUp,
    Globe,
    AlertTriangle,
    Search,
    Terminal,
    Clock,
    Code2,
    LayoutGrid,
    LayoutList,
    CheckCircle2,
    GitCommit,
    ArrowRightLeft,
    Check,
    X,
    FileText,
    Eye
  } from "lucide-svelte";
  import { tick } from "svelte";
  import hljs from 'highlight.js';
  import 'highlight.js/styles/github-dark.css';

  type SyncStatus = 'Clean' | 'Ahead' | 'Dirty' | 'Behind' | 'Diverged';

  interface RepoMetadata {
    name: string;
    path: string;
    description: string | null;
    branch: string;
    sync_status: SyncStatus;
    remote_url: string | null;
    remote_reachable: boolean;
    last_modified: number;
    languages: Record<string, number>;
  }

  let repos = $state<RepoMetadata[]>([]);
  let searchQuery = $state("");
  let selectedLanguages = $state<string[]>([]);
  let viewMode = $state<'grid' | 'list'>('grid');
  let loading = $state(true);
  let isScanning = $state(false);
  let actionLoading = $state<Record<string, string | null>>({});
  let hoveredAction = $state<{ path: string, label: string } | null>(null);
  let showLanguageDropdown = $state(false);
  let selectedRepoForPreview = $state<RepoMetadata | null>(null);
  let readmeContent = $state<{ html: string, raw: string }>({ html: "", raw: "" });
  let readmeLoading = $state(false);
  let previewMode = $state<'markdown' | 'text' | 'unified'>('markdown');
  let unlistenState: UnlistenFn;
  let unlistenStart: UnlistenFn;
  let unlistenEnd: UnlistenFn;

  async function openPreview(repo: RepoMetadata) {
    selectedRepoForPreview = repo;
    readmeLoading = true;
    previewMode = 'markdown';
    try {
      // Backend now returns pre-parsed HTML and RAW content
      const response = await invoke<{ html: string, raw: string }>("get_repo_readme", { path: repo.path });
      readmeContent = response;
      
      // Wait for Svelte to render the HTML before highlighting
      await tick();
      highlightCode();
    } catch (e) {
      readmeContent = { 
        html: "<h3>No README found</h3><p>This repository does not have a standard README file.</p>",
        raw: "No README found" 
      };
    } finally {
      readmeLoading = false;
    }
  }

  function highlightCode() {
    const blocks = document.querySelectorAll('pre code');
    blocks.forEach((block) => {
      hljs.highlightElement(block as HTMLElement);
    });
  }

  // Also highlights when mode changes to markdown
  $effect(() => {
    if (previewMode === 'markdown' && readmeContent.html && !readmeLoading) {
      tick().then(() => highlightCode());
    }
  });

  function closePreview() {
    selectedRepoForPreview = null;
    readmeContent = { html: "", raw: "" };
  }

  // Fuse.js for fuzzy search
  let fuse = $derived(new Fuse(repos, {
    keys: ['name', 'path', 'description'],
    threshold: 0.3
  }));

  const langStats = $derived.by(() => {
    const counts: Record<string, number> = {};
    repos.forEach(repo => {
      Object.keys(repo.languages || {}).forEach(lang => {
        counts[lang] = (counts[lang] || 0) + 1;
      });
    });
    return Object.entries(counts)
      .sort((a, b) => b[1] - a[1])
      .map(([lang, count]) => ({ name: lang, count }));
  });

  const visibleLanguages = $derived(langStats.slice(0, 7));
  const hiddenLanguages = $derived(langStats.slice(7));

  const filteredRepos = $derived.by(() => {
    let result = searchQuery 
      ? fuse.search(searchQuery).map(r => r.item)
      : [...repos];
    
    if (selectedLanguages.length > 0) {
      result = result.filter(r => 
        selectedLanguages.some(lang => Object.keys(r.languages || {}).includes(lang))
      );
    }
    
    return result;
  });

  function toggleLanguage(lang: string) {
    if (selectedLanguages.includes(lang)) {
      selectedLanguages = selectedLanguages.filter(l => l !== lang);
    } else {
      selectedLanguages = [...selectedLanguages, lang];
    }
  }

  function formatRelativeTime(timestamp: number) {
    if (timestamp === 0) return "Never";
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;
    if (diff < 60) return "Just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function getSyncStatusDetails(status: SyncStatus) {
    switch (status) {
      case 'Clean': return { icon: CheckCircle2, color: 'text-emerald-500', label: 'Clean' };
      case 'Ahead': return { icon: ArrowUp, color: 'text-primary', label: 'Ahead' };
      case 'Behind': return { icon: ArrowDown, color: 'text-amber-500', label: 'Behind' };
      case 'Dirty': return { icon: GitCommit, color: 'text-amber-500', label: 'Uncommitted' };
      case 'Diverged': return { icon: ArrowRightLeft, color: 'text-destructive', label: 'Diverged' };
      default: return { icon: AlertTriangle, color: 'text-muted-foreground', label: 'Unknown' };
    }
  }

  async function loadRepos() {
    try {
      repos = await invoke("list_repos");
    } catch (e) {
      console.error("Failed to load repos:", e);
    } finally {
      loading = false;
    }
  }

  async function refreshRepos() {
    if (isScanning) return;
    try {
      await invoke("refresh_repos");
    } catch (e) {
      toast.error("Scan Failed");
    }
  }

  async function runGitAction(repo: RepoMetadata, action: 'fetch' | 'pull' | 'push') {
    const key = `${repo.path}-${action}`;
    actionLoading[key] = action;
    const promise = invoke<string>(`git_${action}`, { path: repo.path });
    toast.promise(promise, {
      loading: `Git ${action}ing ${repo.name}...`,
      success: () => {
        loadRepos();
        return `Git ${action} successful`;
      },
      error: (e: any) => `Git ${action} failed: ${typeof e === 'string' ? e : (e.GitError || e.IoError || "Unknown error")}`,
    });
    try { await promise; } finally { delete actionLoading[key]; }
  }

  async function openFolder(path: string) {
    try {
      await revealItemInDir(path);
    } catch (e) {
      toast.error("Failed to open folder");
    }
  }

  function clickOutside(node: HTMLElement, handler: () => void) {
    const onClick = (event: MouseEvent) => {
      if (node && !node.contains(event.target as Node) && !event.defaultPrevented) {
        handler();
      }
    };
    document.addEventListener('click', onClick, true);
    return {
      destroy() {
        document.removeEventListener('click', onClick, true);
      }
    };
  }

  onMount(async () => {
    await loadRepos();
    isScanning = await invoke("is_scanning");
    unlistenState = await listen("repo-state-changed", () => loadRepos());
    unlistenStart = await listen("scan-started", () => {
      isScanning = true;
    });
    unlistenEnd = await listen("scan-completed", () => {
      isScanning = false;
      loadRepos();
    });
  });

  onDestroy(() => { 
    if (unlistenState) unlistenState();
    if (unlistenStart) unlistenStart();
    if (unlistenEnd) unlistenEnd();
  });
</script>

<div class="p-8 space-y-8 animate-in fade-in duration-500 max-w-7xl mx-auto">
  <!-- Header / Filters -->
  <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
    <div class="space-y-4 flex-1">
      <div>
        <h1 class="text-5xl font-black tracking-tighter text-glow mb-2">Vessels in Dock</h1>
        <p class="text-muted-foreground text-xl">Managing <span class="text-primary font-bold">{repos.length}</span> repositories.</p>
      </div>
      
      <div class="flex flex-wrap items-center gap-3">
        <!-- Fuzzy Search -->
        <div class="relative group min-w-[300px]">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
          <input 
            type="text" 
            bind:value={searchQuery}
            placeholder="Fuzzy search projects..." 
            class="bg-white/5 border border-white/10 rounded-xl pl-10 pr-4 py-2 text-sm w-full focus:outline-none focus:ring-2 focus:ring-primary/40 focus:bg-white/10 transition-all font-medium"
          />
        </div>

        <!-- Language Filter -->
        <div class="flex flex-wrap items-center gap-1.5 bg-white/5 rounded-xl border border-white/10 p-1.5 max-w-full">
          <button 
            onclick={() => selectedLanguages = []}
            class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all border border-transparent {selectedLanguages.length === 0 ? 'bg-primary text-primary-foreground shadow-glow border-primary/20' : 'hover:bg-white/5 text-muted-foreground'}"
          >
            All
          </button>
          
          {#each visibleLanguages as lang}
            <button 
              onclick={() => toggleLanguage(lang.name)}
              class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all border whitespace-nowrap {selectedLanguages.includes(lang.name) ? 'bg-white/10 text-primary border-primary/40 shadow-glow' : 'hover:bg-white/5 text-muted-foreground border-transparent'}"
            >
              <div class="flex items-center gap-2">
                {lang.name}
                <span class="opacity-40 text-[8px] font-medium">{lang.count}</span>
                {#if selectedLanguages.includes(lang.name)}
                  <Check class="w-3 h-3" />
                {/if}
              </div>
            </button>
          {/each}

          {#if hiddenLanguages.length > 0}
            <div class="relative">
              <button 
                onclick={() => showLanguageDropdown = !showLanguageDropdown}
                class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all border whitespace-nowrap {hiddenLanguages.some(l => selectedLanguages.includes(l.name)) ? 'bg-white/10 text-primary border-primary/40' : 'hover:bg-white/5 text-muted-foreground border-transparent'}"
              >
                +{hiddenLanguages.length}
              </button>
              
              {#if showLanguageDropdown}
                <div 
                  use:clickOutside={() => showLanguageDropdown = false}
                  class="absolute top-full right-0 mt-3 w-56 bg-background/98 border border-white/10 rounded-2xl shadow-[0_20px_50px_rgba(0,0,0,0.5)] backdrop-blur-2xl z-[100] p-3 animate-in fade-in zoom-in-95 slide-in-from-top-2 duration-200 ring-1 ring-white/5"
                >
                  <div class="px-3 py-2 mb-2 border-b border-white/5">
                    <span class="text-[9px] font-black uppercase tracking-[0.2em] text-muted-foreground/60">More Languages</span>
                  </div>
                  <div class="max-h-64 overflow-y-auto space-y-1 custom-scrollbar pr-1">
                    {#each hiddenLanguages as lang}
                      <button 
                        onclick={(e) => { e.stopPropagation(); toggleLanguage(lang.name); showLanguageDropdown = false; }}
                        class="w-full text-left px-3 py-2.5 text-[10px] font-black uppercase tracking-widest rounded-xl hover:bg-white/10 transition-all flex items-center justify-between group {selectedLanguages.includes(lang.name) ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground'}"
                      >
                        <div class="flex items-center gap-2 truncate">
                          <span>{lang.name}</span>
                          <span class="opacity-40 text-[8px] font-medium">{lang.count}</span>
                        </div>
                        {#if selectedLanguages.includes(lang.name)}
                          <Check class="w-3.5 h-3.5 text-primary" />
                        {:else}
                          <div class="w-1.5 h-1.5 rounded-full bg-white/5 group-hover:bg-primary/40 transition-colors"></div>
                        {/if}
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </div>

    <div class="flex items-center space-x-3">
      <div class="bg-white/5 rounded-xl border border-white/10 p-1 flex items-center">
        <button 
          onclick={() => viewMode = 'grid'}
          class="p-2 rounded-lg transition-all {viewMode === 'grid' ? 'bg-white/10 text-primary shadow-inner' : 'text-muted-foreground hover:bg-white/5'}"
        >
          <LayoutGrid class="w-4 h-4" />
        </button>
        <button 
          onclick={() => viewMode = 'list'}
          class="p-2 rounded-lg transition-all {viewMode === 'list' ? 'bg-white/10 text-primary shadow-inner' : 'text-muted-foreground hover:bg-white/5'}"
        >
          <LayoutList class="w-4 h-4" />
        </button>
      </div>
      
      <Button variant="outline" size="icon" class="rounded-xl border-white/10" onclick={refreshRepos} disabled={isScanning}>
        <RefreshCw class="w-4 h-4 {isScanning ? 'animate-spin text-primary' : ''}" />
      </Button>
    </div>
  </div>

  {#if loading && repos.length === 0}
    <!-- Loading state... -->
    <div class="py-32 flex flex-col items-center justify-center space-y-6">
      <div class="relative w-16 h-16">
        <div class="absolute inset-0 border-4 border-primary/20 rounded-full"></div>
        <div class="absolute inset-0 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
      </div>
      <p class="text-sm font-black uppercase tracking-[0.2em] text-primary/60 animate-pulse">Scanning Horizon</p>
    </div>
  {:else if filteredRepos.length === 0}
    <div class="py-32 glass rounded-3xl flex flex-col items-center justify-center space-y-8 text-center px-6">
      <div class="relative p-8 rounded-full bg-white/5 border border-white/10">
        <Search class="w-16 h-16 text-primary opacity-20" />
      </div>
      <div class="space-y-3">
        <h3 class="text-3xl font-bold">No Vessels Found</h3>
        <p class="text-muted-foreground max-w-sm mx-auto text-lg">
          No repositories match your search or filter criteria.
        </p>
      </div>
      <Button variant="outline" class="rounded-full px-8" onclick={() => { searchQuery = ""; selectedLanguages = []; }}>Reset Filters</Button>
    </div>
  {:else}
    <div class={viewMode === 'grid' ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6" : "space-y-3"}>
      {#each filteredRepos as repo (repo.path)}
        {@const status = getSyncStatusDetails(repo.sync_status)}
        <Card 
          class="group glass glass-hover border-white/5 flex flex-col rounded-[1.5rem] overflow-hidden transition-all duration-500 {viewMode === 'list' ? 'flex-row items-center py-2 px-6' : ''} cursor-pointer {selectedRepoForPreview?.path === repo.path ? 'ring-2 ring-primary border-primary/40 shadow-glow bg-primary/5' : ''}"
          onclick={(e) => {
            if (e.target.closest('button')) return;
            openPreview(repo);
          }}
        >
          {#if viewMode === 'grid'}
            <!-- Grid Item -->
            <CardContent class="p-8 space-y-8 flex-1 flex flex-col">
              <div class="flex items-start justify-between gap-6">
                <div class="space-y-2 flex-1 min-w-0">
                  <h3 class="text-2xl font-black tracking-tight truncate group-hover:text-primary transition-colors">{repo.name}</h3>
                  <button 
                    onclick={() => openFolder(repo.path)}
                    class="text-[10px] bg-white/5 border border-white/5 hover:border-white/20 px-2 py-0.5 rounded-full font-mono text-muted-foreground truncate max-w-full block hover:text-foreground transition-all transition-all"
                    title={repo.path}
                  >
                    {repo.path}
                  </button>
                </div>
                <div class="p-3 bg-white/5 rounded-2xl text-muted-foreground group-hover:text-primary group-hover:bg-primary/10 transition-all duration-500">
                  <GitBranch class="w-5 h-5" />
                </div>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div class="bg-white/5 rounded-2xl p-3 border border-white/5 hover:border-white/10 transition-all">
                  <div class="flex items-center space-x-2 mb-1">
                    <GitBranch class="w-3 h-3 text-muted-foreground" />
                    <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground">Branch</span>
                  </div>
                  <p class="text-xs font-bold truncate">{repo.branch}</p>
                </div>
                <div class="bg-white/5 rounded-2xl p-3 border border-white/5 hover:border-white/10 transition-all">
                  <div class="flex items-center space-x-2 mb-1">
                    <status.icon class="w-3 h-3 {status.color}" />
                    <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground">Status</span>
                  </div>
                  <p class="text-xs font-bold {status.color}">{status.label}</p>
                </div>
              </div>

              <div class="flex flex-col gap-3">
                <div class="flex flex-wrap gap-1.5 min-h-[24px]">
                  {#each Object.entries(repo.languages || {}).sort((a, b) => b[1] - a[1]).slice(0, 3) as [lang, count]}
                    <Badge variant="outline" class="bg-white/5 border-white/5 text-[8px] px-2 py-0.5 rounded-md font-black uppercase tracking-widest text-muted-foreground/80">
                      {lang}
                    </Badge>
                  {/each}
                  {#if Object.keys(repo.languages || {}).length > 3}
                    <Badge variant="outline" class="bg-white/5 border-white/5 text-[8px] px-2 py-0.5 rounded-md font-black">
                      +{Object.keys(repo.languages).length - 3}
                    </Badge>
                  {/if}
                </div>

                {#if repo.description}
                  <p class="text-[13px] text-muted-foreground/90 line-clamp-3 min-h-[3rem] leading-relaxed font-medium">
                    {repo.description}
                  </p>
                {/if}
              </div>

              <div class="flex flex-wrap gap-2 pt-2">
                <Badge variant="outline" class="bg-white/5 border-white/5 text-[10px] px-3 py-1 rounded-full font-bold">
                  <Clock class="w-3 h-3 mr-1.5 text-primary" />
                  {formatRelativeTime(repo.last_modified)}
                </Badge>
                {#if repo.remote_url}
                  <Badge variant="outline" class="bg-white/5 border-white/5 text-[10px] px-3 py-1 rounded-full font-bold {repo.remote_reachable ? 'text-emerald-500' : 'text-amber-500 opacity-80'}">
                    <Globe class="w-3 h-3 mr-1.5" />
                    {repo.remote_reachable ? 'Connected' : 'Unreachable'}
                  </Badge>
                {/if}
              </div>

              <div class="mt-auto pt-8 flex items-center justify-between border-t border-white/5 relative">
                <div class="flex items-center bg-black/40 rounded-full p-1.5 border border-white/5 shadow-inner">
                  <Button 
                    variant="ghost" size="icon" class="rounded-full h-11 w-11 hover:bg-white/5" 
                    onmouseenter={() => hoveredAction = { path: repo.path, label: 'Fetch' }}
                    onmouseleave={() => hoveredAction = null}
                    disabled={!!actionLoading[`${repo.path}-fetch`]}
                    onclick={() => runGitAction(repo, 'fetch')}
                  >
                    <RefreshCw class="w-5 h-5 {actionLoading[`${repo.path}-fetch`] ? 'animate-spin' : ''}" />
                  </Button>
                  <Button 
                    variant="ghost" size="icon" class="rounded-full h-11 w-11 hover:bg-white/5" 
                    onmouseenter={() => hoveredAction = { path: repo.path, label: 'Pull' }}
                    onmouseleave={() => hoveredAction = null}
                    disabled={!!actionLoading[`${repo.path}-pull`]}
                    onclick={() => runGitAction(repo, 'pull')}
                  >
                    <ArrowDown class="w-5 h-5 {actionLoading[`${repo.path}-pull`] ? 'animate-bounce' : ''}" />
                  </Button>
                  <Button 
                    variant="ghost" size="icon" class="rounded-full h-11 w-11 hover:bg-white/5" 
                    onmouseenter={() => hoveredAction = { path: repo.path, label: 'Push' }}
                    onmouseleave={() => hoveredAction = null}
                    disabled={!!actionLoading[`${repo.path}-push`]}
                    onclick={() => runGitAction(repo, 'push')}
                  >
                    <ArrowUp class="w-5 h-5 {actionLoading[`${repo.path}-push`] ? 'animate-pulse text-primary' : ''}" />
                  </Button>
                </div>

                {#if hoveredAction?.path === repo.path}
                  <div class="absolute -top-6 left-1/2 -translate-x-1/2 bg-primary px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest text-primary-foreground shadow-glow animate-in fade-in slide-in-from-bottom-1 duration-200 z-10">
                    {hoveredAction.label}
                  </div>
                {/if}

                <div class="flex items-center space-x-1">
                  <Button variant="ghost" size="icon" class="rounded-full hover:bg-white/5" onmouseenter={() => hoveredAction = { path: repo.path, label: 'Explore' }} onmouseleave={() => hoveredAction = null} onclick={() => openFolder(repo.path)}>
                    <FolderOpen class="w-5 h-5" />
                  </Button>
                </div>
              </div>
            </CardContent>
          {:else}
            <!-- List Item -->
            <div class="flex-1 flex items-center justify-between py-2.5 px-6 overflow-hidden gap-8">
              <div class="flex items-center space-x-5 min-w-0 flex-1">
                <div class="p-2.5 bg-white/5 rounded-2xl text-muted-foreground/60 group-hover:text-primary transition-all duration-500 group-hover:bg-primary/10">
                  <GitBranch class="w-5 h-5" />
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center space-x-3 mb-0.5">
                    <h3 class="font-bold truncate text-sm tracking-tight">{repo.name}</h3>
                    <div class="flex gap-1">
                      {#each Object.keys(repo.languages || {}).slice(0, 2) as lang}
                        <span class="text-[8px] px-1.5 py-0.5 bg-white/5 border border-white/5 text-muted-foreground font-black uppercase tracking-widest rounded-md">{lang}</span>
                      {/each}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button 
                      onclick={() => openFolder(repo.path)}
                      class="text-[10px] text-muted-foreground/60 font-mono truncate hover:text-primary transition-colors block text-left"
                    >
                      {repo.path}
                    </button>
                    {#if repo.description}
                       <span class="text-[10px] text-muted-foreground/40 font-medium truncate italic">• {repo.description}</span>
                    {/if}
                  </div>
                </div>
              </div>

              <div class="flex items-center space-x-16 flex-shrink-0">
                <div class="flex flex-col items-center">
                  <span class="text-[9px] uppercase tracking-[0.25em] font-black text-muted-foreground/40 mb-1.5">Branch</span>
                  <span class="text-sm font-bold tracking-tight">{repo.branch}</span>
                </div>
                <div class="flex flex-col items-center">
                  <span class="text-[9px] uppercase tracking-[0.25em] font-black text-muted-foreground/40 mb-1.5">Status</span>
                  <div class="flex items-center space-x-2">
                    <status.icon class="w-4 h-4 {status.color}" />
                    <span class="text-sm font-bold tracking-tight {status.color}">{status.label}</span>
                  </div>
                </div>
                <div class="flex flex-col items-center">
                  <span class="text-[9px] uppercase tracking-[0.25em] font-black text-muted-foreground/40 mb-1.5">Activity</span>
                  <span class="text-sm font-bold text-muted-foreground/80 tracking-tight">{formatRelativeTime(repo.last_modified)}</span>
                </div>
              </div>

              <div class="flex items-center space-x-2 flex-shrink-0 relative">
                <Button variant="ghost" size="icon" class="rounded-xl h-11 w-11 hover:bg-white/5 hover:text-primary transition-all" onmouseenter={() => hoveredAction = { path: repo.path, label: 'Inspect' }} onmouseleave={() => hoveredAction = null} onclick={() => openPreview(repo)}>
                  <Eye class="w-5 h-5" />
                </Button>
                <div class="h-8 w-px bg-white/5 mx-1"></div>
                <Button 
                   variant="ghost" size="icon" class="rounded-xl h-11 w-11 hover:bg-white/5 hover:text-emerald-500" 
                   onmouseenter={() => hoveredAction = { path: repo.path, label: 'Fetch' }}
                   onmouseleave={() => hoveredAction = null}
                   disabled={!!actionLoading[`${repo.path}-fetch`]}
                   onclick={() => runGitAction(repo, 'fetch')}
                >
                  <RefreshCw class="w-5 h-5 {actionLoading[`${repo.path}-fetch`] ? 'animate-spin' : ''}" />
                </Button>
                <Button 
                   variant="ghost" size="icon" class="rounded-xl h-11 w-11 hover:bg-white/5 hover:text-primary" 
                   onmouseenter={() => hoveredAction = { path: repo.path, label: 'Pull' }}
                   onmouseleave={() => hoveredAction = null}
                   disabled={!!actionLoading[`${repo.path}-pull`]}
                   onclick={() => runGitAction(repo, 'pull')}
                >
                  <ArrowDown class="w-5 h-5 {actionLoading[`${repo.path}-pull`] ? 'animate-bounce' : ''}" />
                </Button>
                <Button 
                   variant="ghost" size="icon" class="rounded-xl h-11 w-11 hover:bg-white/5 hover:text-blue-500" 
                   onmouseenter={() => hoveredAction = { path: repo.path, label: 'Finder' }}
                   onmouseleave={() => hoveredAction = null}
                   onclick={() => openFolder(repo.path)}
                >
                  <FolderOpen class="w-5 h-5" />
                </Button>

                {#if hoveredAction?.path === repo.path}
                  <div class="absolute -top-12 right-0 bg-primary px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest text-primary-foreground shadow-glow animate-in fade-in slide-in-from-right-1 duration-200 z-10 whitespace-nowrap">
                    {hoveredAction.label}
                  </div>
                {/if}
              </div>
            </div>
          {/if}
        </Card>
      {/each}
    </div>
  {/if}
</div>

<!-- Side Panel (README Preview) -->
{#if selectedRepoForPreview}
  <div 
    class="fixed inset-0 bg-black/40 backdrop-blur-sm z-[100] transition-all duration-300"
    onclick={closePreview}
    aria-hidden="true"
  ></div>
  <div 
    class="fixed top-0 right-0 h-full w-[45%] bg-background border-l border-white/10 z-[101] flex flex-col shadow-2xl animate-in slide-in-from-right duration-500 ease-out"
  >
    <div class="p-6 border-b border-white/5 flex items-center justify-between bg-white/5">
      <div class="space-y-1">
        <h2 class="text-xl font-black uppercase tracking-widest">{selectedRepoForPreview.name}</h2>
        <p class="text-[10px] text-muted-foreground font-mono">{selectedRepoForPreview.path}</p>
      </div>
      <Button variant="ghost" size="icon" class="rounded-xl hover:bg-white/10" onclick={closePreview}>
        <X class="w-5 h-5" />
      </Button>
    </div>

    <!-- Mode Selector -->
    <div class="px-6 py-4 flex items-center justify-between border-b border-white/5 bg-white/[0.02]">
      <div class="flex items-center space-x-2 bg-white/5 p-1 rounded-xl border border-white/10">
        <button 
          onclick={() => previewMode = 'markdown'}
          class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-lg transition-all {previewMode === 'markdown' ? 'bg-primary text-primary-foreground shadow-glow' : 'text-muted-foreground hover:text-foreground'}"
        >
          Markdown
        </button>
        <button 
          onclick={() => previewMode = 'text'}
          class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-lg transition-all {previewMode === 'text' ? 'bg-primary text-primary-foreground shadow-glow' : 'text-muted-foreground hover:text-foreground'}"
        >
          Raw
        </button>
        <button 
          onclick={() => previewMode = 'unified'}
          class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-lg transition-all {previewMode === 'unified' ? 'bg-primary text-primary-foreground shadow-glow' : 'text-muted-foreground hover:text-foreground'}"
        >
          Unified
        </button>
      </div>
      
      <div class="flex items-center gap-2">
         <Button variant="outline" size="sm" class="h-8 rounded-lg border-white/10 text-[10px] font-bold uppercase tracking-widest bg-white/5 hover:bg-white/10" onclick={() => openFolder(selectedRepoForPreview.path)}>
           <FolderOpen class="w-3 h-3 mr-2 text-primary" />
           Open Folder
         </Button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-12 prose prose-invert prose-lg prose-neutral max-w-none no-scrollbar 
      prose-a:text-primary prose-a:no-underline hover:prose-a:underline
      prose-blockquote:border-l-primary prose-blockquote:bg-primary/5 prose-blockquote:rounded-r-2xl prose-blockquote:py-2
      prose-img:rounded-3xl prose-img:border prose-img:border-white/10 prose-img:shadow-2xl">
      {#if readmeLoading}
        <div class="h-full flex flex-col items-center justify-center space-y-6 py-48">
          <div class="relative w-16 h-16">
            <RefreshCw class="w-full h-full animate-spin text-primary opacity-20" />
            <div class="absolute inset-0 flex items-center justify-center">
              <FileText class="w-6 h-6 text-primary" />
            </div>
          </div>
          <p class="text-xs font-black uppercase tracking-[0.4em] text-muted-foreground animate-pulse">Decrypting Repository Core</p>
        </div>
      {:else}
        {#if previewMode === 'markdown'}
          <div class="animate-in fade-in slide-in-from-bottom-4 duration-1000">
            {@html readmeContent.html}
          </div>
        {:else if previewMode === 'text'}
          <pre class="bg-black/40 p-10 rounded-[2.5rem] border border-white/10 font-mono text-[15px] leading-relaxed text-muted-foreground/90 whitespace-pre-wrap animate-in fade-in duration-500 shadow-2xl overflow-x-auto">
            {readmeContent.raw}
          </pre>
        {:else}
          <div class="space-y-10 animate-in fade-in duration-500 pb-20">
            <div class="space-y-4">
              <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Metadata Summary</h3>
              <div class="grid grid-cols-2 gap-4">
                 <div class="bg-white/5 p-5 rounded-2xl border border-white/5">
                   <p class="text-[9px] text-muted-foreground uppercase font-black mb-1.5 tracking-widest">Active Branch</p>
                   <div class="flex items-center gap-2.5">
                     <div class="p-1.5 bg-primary/10 rounded-lg text-primary">
                       <GitBranch class="w-4 h-4" />
                     </div>
                     <span class="text-sm font-bold tracking-tight">{selectedRepoForPreview.branch}</span>
                   </div>
                 </div>
                 <div class="bg-white/5 p-5 rounded-2xl border border-white/5">
                   <p class="text-[9px] text-muted-foreground uppercase font-black mb-1.5 tracking-widest">Dock Progress</p>
                   <div class="flex items-center gap-2.5">
                     <div class="p-1.5 bg-amber-500/10 rounded-lg text-amber-500">
                       <Clock class="w-4 h-4" />
                     </div>
                     <span class="text-sm font-bold tracking-tight">{getSyncStatusDetails(selectedRepoForPreview.sync_status).label}</span>
                   </div>
                 </div>
              </div>
            </div>
            
            <div class="space-y-4">
               <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Manifested Artifacts</h3>
               <div class="flex flex-wrap gap-2.5">
                 {#each Object.entries(selectedRepoForPreview.languages).sort((a, b) => b[1] - a[1]) as [lang, count]}
                   <div class="bg-white/[0.03] px-5 py-3 rounded-2xl border border-white/5 flex items-center gap-4 hover:border-primary/20 transition-colors">
                     <div class="w-2 h-2 rounded-full bg-primary/40"></div>
                     <span class="text-xs font-bold uppercase tracking-wider">{lang}</span>
                     <span class="text-[9px] font-black p-1 bg-white/5 rounded border border-white/5 text-muted-foreground">{count}</span>
                   </div>
                 {/each}
               </div>
            </div>

            {#if selectedRepoForPreview.description}
              <div class="space-y-4">
                 <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Intelligence</h3>
                 <div class="bg-primary/5 p-6 rounded-3xl border border-primary/10 italic text-sm text-muted-foreground leading-relaxed">
                   "{selectedRepoForPreview.description}"
                 </div>
              </div>
            {/if}
            
            <div class="space-y-4">
               <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Remote Link</h3>
               <div class="bg-white/5 p-5 rounded-2xl border border-white/5 flex items-center justify-between">
                 <div class="flex items-center gap-3">
                    <Globe class="w-4 h-4 text-muted-foreground" />
                    <span class="text-xs font-mono text-muted-foreground truncate max-w-xs">{selectedRepoForPreview.remote_url || 'No Remote Configured'}</span>
                 </div>
                 {#if selectedRepoForPreview.remote_url}
                    <Badge variant="outline" class="{selectedRepoForPreview.remote_reachable ? 'bg-emerald-500/10 text-emerald-500' : 'bg-destructive/10 text-destructive'} border-transparent">
                      {selectedRepoForPreview.remote_reachable ? 'Online' : 'Offline'}
                    </Badge>
                 {/if}
               </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </div>
{/if}

