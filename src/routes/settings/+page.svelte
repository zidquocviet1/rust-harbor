<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card";
  import { Separator } from "$lib/components/ui/separator";
  import { toast } from "svelte-sonner";
  import { 
    FolderPlus, 
    Trash2, 
    Home, 
    Settings, 
    ChevronLeft,
    Shield,
    Database,
    Binary,
    Zap
  } from "lucide-svelte";

  let watchedFolders = $state<string[]>([]);
  let loading = $state(true);

  async function loadConfig() {
    try {
      const config = await invoke<any>("get_config");
      watchedFolders = config.watched_folders || [];
    } catch (e) {
      console.error("Failed to load config:", e);
      toast.error("Configuration Error", { description: "Failed to read application settings." });
    } finally {
      loading = false;
    }
  }

  async function saveConfig() {
    try {
      await invoke("set_config", { config: { watched_folders: watchedFolders } });
    } catch (e) {
      console.error("Failed to save config:", e);
      toast.error("Save Failed", { description: "Changes could not be persisted to disk." });
    }
  }

  async function addFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a Workspace Folder"
    });

    if (selected && typeof selected === "string") {
      if (!watchedFolders.includes(selected)) {
        watchedFolders = [...watchedFolders, selected];
        await saveConfig();
        await invoke("refresh_repos");
        toast.success("Folder Added", { description: "Harbor will now scan this directory." });
      } else {
        toast.info("Folder Exists", { description: "This directory is already being watched." });
      }
    }
  }

  async function removeFolder(path: string) {
    watchedFolders = watchedFolders.filter(f => f !== path);
    await saveConfig();
    await invoke("refresh_repos");
    toast.success("Folder Removed");
  }

  onMount(loadConfig);
</script>

<div class="p-8 space-y-8 animate-in fade-in duration-500 max-w-5xl mx-auto">
  <div class="space-y-2">
    <h2 class="text-4xl font-black tracking-tight text-glow">System Settings</h2>
    <p class="text-muted-foreground text-lg">Define your workspaces and manage internal preferences.</p>
  </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
      <!-- Left: Navigation / Categories -->
      <div class="space-y-3">
        <Button variant="secondary" class="w-full justify-start rounded-xl py-6 bg-primary/10 text-primary border border-primary/20 shadow-lg shadow-primary/5">
          <Database class="w-4 h-4 mr-3" />
          Watched Folders
        </Button>
        <Button variant="ghost" class="w-full justify-start rounded-xl py-6 opacity-40 cursor-not-allowed">
          <Zap class="w-4 h-4 mr-3" />
          Performance
        </Button>
        <Button variant="ghost" class="w-full justify-start rounded-xl py-6 opacity-40 cursor-not-allowed">
          <Shield class="w-4 h-4 mr-3" />
          Security
        </Button>
        <Button variant="ghost" class="w-full justify-start rounded-xl py-6 opacity-40 cursor-not-allowed">
          <Binary class="w-4 h-4 mr-3" />
          Git Binary
        </Button>
      </div>

      <!-- Right: Content Area -->
      <div class="md:col-span-2 space-y-8 animate-in fade-in slide-in-from-right-5 duration-500">
        <Card class="glass border-white/5 overflow-hidden">
          <CardHeader class="pb-6 border-b border-white/5 bg-white/[0.02]">
            <div class="flex items-center justify-between">
              <div class="space-y-1">
                <CardTitle class="text-xl font-bold">Watched Folders</CardTitle>
                <CardDescription class="text-sm">
                  Harbor recursively scans these locations for Git repositories.
                </CardDescription>
              </div>
              <Button size="sm" onclick={addFolder} class="rounded-full bg-primary hover:scale-105 transition-transform px-4">
                <FolderPlus class="w-4 h-4 mr-2" />
                Add Folder
              </Button>
            </div>
          </CardHeader>
          <CardContent class="pt-6">
            {#if loading}
              <div class="py-12 flex justify-center">
                <div class="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
              </div>
            {:else if watchedFolders.length === 0}
              <div class="py-20 border-2 border-dashed border-white/5 rounded-2xl flex flex-col items-center justify-center space-y-4 text-muted-foreground">
                <div class="p-4 rounded-full bg-white/5">
                  <FolderPlus class="w-8 h-8 opacity-20" />
                </div>
                <div class="text-center">
                  <p class="font-semibold text-foreground">No folders tracked</p>
                  <p class="text-xs">Add a directory to begin monitoring repos.</p>
                </div>
              </div>
            {:else}
              <div class="space-y-3">
                {#each watchedFolders as folder}
                  <div class="group flex items-center justify-between p-4 rounded-xl border border-white/5 bg-white/[0.02] hover:bg-white/[0.05] hover:border-white/10 transition-all duration-300">
                    <div class="flex items-center space-x-3 min-w-0">
                      <div class="p-2 rounded-lg bg-white/5 text-muted-foreground group-hover:text-primary transition-colors">
                        <Database class="w-4 h-4" />
                      </div>
                      <span class="text-sm font-mono truncate text-muted-foreground group-hover:text-foreground transition-colors leading-none">
                        {folder}
                      </span>
                    </div>
                    <Button 
                      variant="ghost" 
                      size="icon" 
                      class="rounded-full h-9 w-9 opacity-0 group-hover:opacity-100 text-destructive hover:bg-destructive/10 transition-all" 
                      onclick={() => removeFolder(folder)}
                    >
                      <Trash2 class="w-4 h-4" />
                    </Button>
                  </div>
                {/each}
              </div>
            {/if}
          </CardContent>
        </Card>

        <div class="glass p-6 rounded-2xl border-amber-500/20 bg-amber-500/5 flex items-start space-x-4">
          <div class="p-2 rounded-lg bg-amber-500/20 text-amber-500">
            <Shield class="w-5 h-5" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-bold text-amber-500">Privacy Notice</p>
            <p class="text-xs text-amber-200/60 leading-relaxed">
              Harbor only communicates with your local Git binary. Your source code and repository data never leave your machine.
            </p>
          </div>
      </div>
    </div>
</div>
</div>
