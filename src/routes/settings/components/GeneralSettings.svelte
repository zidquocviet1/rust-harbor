<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card";
  import { toast } from "svelte-sonner";
  import { FolderPlus, Trash2, Database, AlertCircle, RefreshCw, ChevronRight, ChevronDown } from "lucide-svelte";
  import { APP_BRANDING } from "$lib/config/branding";

  interface Insight {
    path: string;
    repo_count: number;
    last_scan_time: number | null;
    scan_status: string;
    error_details: string | null;
  }

  interface TreeNode {
    folder: Insight;
    children: TreeNode[];
  }

  let insights = $state<Insight[]>([]);
  let loading = $state(true);
  let tree = $derived(buildTree(insights));
  let expandedPaths = $state<Set<string>>(new Set());

  function toggleExpand(path: string) {
    const next = new Set(expandedPaths);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedPaths = next;
  }

  async function loadInsights() {
    try {
      loading = true;
      insights = await invoke("get_workspace_insights");
      // Initially expand all
      insights.forEach(i => expandedPaths.add(i.path));
    } catch (e) {
      console.error("Failed to load insights:", e);
    } finally {
      loading = false;
    }
  }

  function buildTree(flatInsights: Insight[]): TreeNode[] {
    const sorted = [...flatInsights].sort((a, b) => a.path.length - b.path.length);
    const roots: TreeNode[] = [];
    const map = new Map<string, TreeNode>();

    for (const folder of sorted) {
      const node: TreeNode = { folder, children: [] };
      map.set(folder.path, node);

      let parentPath = null;
      const parts = folder.path.split('/');
      for (let i = parts.length - 1; i > 0; i--) {
        const potentialParent = parts.slice(0, i).join('/');
        if (map.has(potentialParent)) {
          parentPath = potentialParent;
          break;
        }
      }

      if (parentPath && map.get(parentPath)) {
        map.get(parentPath)!.children.push(node);
      } else {
        roots.push(node);
      }
    }
    return roots;
  }

  async function addFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a Workspace Folder"
    });

    if (selected && typeof selected === "string") {
      try {
        const config = await invoke<any>("get_config");
        if (!config.watched_folders.includes(selected)) {
          config.watched_folders.push(selected);
          await invoke("set_config", { config });
          await invoke("refresh_repos");
          toast.success("Folder Added");
          await loadInsights();
        } else {
          toast.info("Folder Exists");
        }
      } catch (e) {
        toast.error("Action Failed");
      }
    }
  }

  async function removeFolder(path: string) {
    try {
       const config = await invoke<any>("get_config");
       config.watched_folders = config.watched_folders.filter((f: string) => f !== path);
       await invoke("set_config", { config });
       await invoke("refresh_repos");
       toast.success("Folder Removed");
       await loadInsights();
    } catch (e) {
       toast.error("Action Failed");
    }
  }

  onMount(loadInsights);
</script>

{#snippet treeNode(node: TreeNode, depth: number)}
  {@const isExpanded = expandedPaths.has(node.folder.path)}
  <div class="space-y-4">
    <div 
      class="group flex flex-col p-4 rounded-2xl border border-white/5 bg-white/[0.02] hover:bg-white/[0.04] hover:border-white/10 transition-all duration-500"
      style="margin-left: {depth * 24}px"
    >
      <div class="flex items-center justify-between">
         <div class="flex items-center space-x-3 min-w-0 flex-1">
            {#if node.children.length > 0}
              <button 
                type="button"
                onclick={() => toggleExpand(node.folder.path)}
                class="p-1 hover:bg-white/10 rounded-md transition-colors"
                aria-label={isExpanded ? "Collapse" : "Expand"}
              >
                {#if isExpanded}
                  <ChevronDown class="w-4 h-4 text-muted-foreground/40" />
                {:else}
                  <ChevronRight class="w-4 h-4 text-muted-foreground/40" />
                {/if}
              </button>
            {:else if depth > 0}
              <div class="w-6 h-6"></div>
            {/if}
            <div class="p-2.5 rounded-xl bg-white/5 text-muted-foreground group-hover:text-primary group-hover:bg-primary/5 transition-all duration-500">
              <Database class="w-5 h-5" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <p class="text-sm font-semibold truncate text-foreground/90 group-hover:text-foreground transition-colors leading-tight">
                  {node.folder.path.split('/').pop() || node.folder.path}
                </p>
                <div class="w-2 h-2 rounded-full {node.folder.scan_status === 'Synced' ? 'bg-emerald-500 box-shadow-emerald' : 'bg-amber-500 box-shadow-amber animate-pulse'}"></div>
              </div>
              <p class="text-[10px] font-mono opacity-40 truncate">
                {node.folder.path}
              </p>
            </div>
         </div>
         <div class="flex items-center gap-3">
            <Button 
              variant="ghost" 
              size="icon" 
              class="rounded-full h-8 w-8 text-destructive opacity-0 group-hover:opacity-100 hover:bg-destructive/10 transition-all" 
              onclick={() => removeFolder(node.folder.path)}
            >
              <Trash2 class="w-4 h-4" />
            </Button>
         </div>
      </div>
      
      {#if node.folder.error_details}
        <div class="flex items-center text-amber-500/80 text-[10px] italic mt-2 pt-2 border-t border-white/5">
          <AlertCircle class="w-3 h-3 mr-1" />
          Hub unreachable: {node.folder.error_details}
        </div>
      {/if}
    </div>
    
    {#if isExpanded && node.children.length > 0}
      {#each node.children as child}
        {@render treeNode(child, depth + 1)}
      {/each}
    {/if}
  </div>
{/snippet}

<Card class="glass border-white/5 overflow-hidden">
  <CardHeader class="pb-6 border-b border-white/5 bg-white/[0.02]">
    <div class="flex items-center justify-between">
      <div class="space-y-1">
        <CardTitle class="text-xl font-bold">Workspace Hub</CardTitle>
        <CardDescription class="text-sm">
          Monitor and manage your Git workspace roots.
        </CardDescription>
      </div>
      <div class="flex items-center gap-2">
        <Button variant="ghost" size="icon" class="rounded-full" onclick={loadInsights}>
          <RefreshCw class="w-4 h-4 {loading ? 'animate-spin' : ''}" />
        </Button>
        <Button size="sm" onclick={addFolder} class="rounded-full bg-primary hover:scale-105 transition-transform px-4 border shadow-lg shadow-primary/20">
          <FolderPlus class="w-4 h-4 mr-2" />
          Add Hub
        </Button>
      </div>
    </div>
  </CardHeader>
  <CardContent class="pt-6">
    <!-- Status Legend -->
    <div class="flex items-center gap-6 mb-8 px-2">
      <div class="flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-emerald-500 box-shadow-emerald"></div>
        <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Synced</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="w-2 h-2 rounded-full bg-amber-500 box-shadow-amber animate-pulse"></div>
        <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">Scanning / Warning</span>
      </div>
    </div>

    {#if loading && insights.length === 0}
      <div class="py-12 flex justify-center">
        <div class="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
      </div>
    {:else if insights.length === 0}
      <div class="py-20 border-2 border-dashed border-white/5 rounded-2xl flex flex-col items-center justify-center space-y-4 text-muted-foreground bg-white/[0.01]">
        <div class="p-4 rounded-full bg-white/5">
          <Database class="w-8 h-8 opacity-20" />
        </div>
        <div class="text-center">
          <p class="font-semibold text-foreground">No hubs tracked</p>
          <p class="text-xs">Add a directory to start discovery.</p>
        </div>
      </div>
    {:else}
      <div class="space-y-6">
        {#each tree as root}
          {@render treeNode(root, 0)}
        {/each}
      </div>
    {/if}
  </CardContent>
</Card>

<style>
  .box-shadow-emerald {
    box-shadow: 0 0 8px rgba(16, 185, 129, 0.4);
  }
  .box-shadow-amber {
    box-shadow: 0 0 8px rgba(245, 158, 11, 0.4);
  }
</style>
