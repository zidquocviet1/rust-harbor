<script lang="ts">
  import { AlertCircle, Terminal, Download, RefreshCw, Binary, Save } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { APP_BRANDING } from "$lib/config/branding";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  let { currentPath = "git" } = $props();
  let newPath = $state(currentPath);
  let loading = $state(false);

  async function updatePath() {
    loading = true;
    try {
      // Fetch latest config to merge
      const config = await invoke<any>("get_config");
      config.git_path = newPath;
      await invoke("set_config", { config });
      toast.success("Path updated. Retrying...");
      
      // The layout listener will pick up the 'config-changed' event
      // but we can also manually trigger a reload or re-check if needed.
      // For now, let's rely on the layout listener or the retry button.
    } catch (e) {
      toast.error("Failed to update path");
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    newPath = currentPath;
  });
</script>

<div class="fixed inset-0 z-[200] flex items-center justify-center p-6 overflow-hidden">
  <!-- Backdrop Blur -->
  <div class="absolute inset-0 bg-background/40 backdrop-blur-2xl"></div>
  
  <!-- Ambient Error Glow -->
  <div class="absolute w-[500px] h-[500px] bg-destructive/10 rounded-full blur-[120px] animate-pulse"></div>

  <div class="relative max-w-2xl w-full">
    <div class="glass border-destructive/20 rounded-[2.5rem] p-12 space-y-10 shadow-[0_0_80px_-15px_rgba(239,68,68,0.25)] animate-in fade-in zoom-in duration-700">
      <div class="flex flex-col items-center text-center space-y-6">
        <div class="relative group">
          <div class="absolute inset-0 bg-destructive/30 rounded-3xl blur-2xl group-hover:blur-3xl transition-all duration-700"></div>
          <div class="relative p-7 bg-destructive/10 border border-destructive/20 rounded-3xl text-destructive shadow-inner">
            <AlertCircle class="w-16 h-16" />
          </div>
        </div>
        
        <div class="space-y-3">
          <h1 class="text-5xl font-black tracking-tighter text-glow text-destructive uppercase">Core Failure</h1>
          <p class="text-muted-foreground text-xl leading-relaxed max-w-md mx-auto font-medium">
             Git binary not found at the specified location.
          </p>
        </div>
      </div>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div class="flex flex-col p-6 rounded-3xl bg-white/5 border border-white/5 space-y-4">
           <div class="flex items-center space-x-3 text-primary">
              <Binary class="w-5 h-5" />
              <span class="text-sm font-bold uppercase tracking-widest">Active Search Path</span>
           </div>
           <div class="space-y-4">
              <div class="relative">
                <Input 
                  bind:value={newPath} 
                  class="bg-white/[0.03] border-white/20 hover:border-white/40 focus:border-primary/50 rounded-2xl h-12 pl-4 pr-12 font-mono text-sm transition-all shadow-inner"
                  placeholder="e.g. /usr/bin/git"
                />
                <Button 
                  size="icon" 
                  variant="ghost" 
                  class="absolute right-1.5 top-1.5 h-9 w-9 rounded-xl hover:bg-white/10"
                  onclick={updatePath}
                  disabled={loading}
                >
                  <Save class="w-4 h-4 {loading ? 'animate-pulse' : ''}" />
                </Button>
              </div>
              <p class="text-[10px] text-muted-foreground leading-relaxed px-1">
                Currently looking for Git at <span class="text-primary font-bold font-mono px-1.5 py-0.5 rounded bg-primary/10 border border-primary/20">{currentPath}</span>. If Git is installed in a non-standard path, update it above.
              </p>
           </div>
        </div>

        <div class="flex flex-col p-6 rounded-3xl bg-white/5 border border-white/5 space-y-4 justify-between">
           <div class="flex items-center space-x-3 text-emerald-500">
              <Download class="w-5 h-5" />
              <span class="text-sm font-bold uppercase tracking-widest">Environment Help</span>
           </div>
           <p class="text-xs text-muted-foreground leading-relaxed">
             Download the official Git CLI for your system to ensure full compatibility with {APP_BRANDING.shortName}.
           </p>
           <a 
            href="https://git-scm.com" 
            target="_blank" 
            class="inline-flex items-center justify-center h-10 px-4 rounded-xl bg-emerald-500/10 text-emerald-500 text-xs font-bold hover:bg-emerald-500/20 transition-colors"
           >
             Official Downloads
           </a>
        </div>
      </div>
      
      <div class="pt-2">
        <Button 
          variant="destructive"
          size="lg"
          onclick={() => window.location.reload()}
          class="w-full py-8 rounded-[1.5rem] text-xl font-black uppercase tracking-[0.1em] shadow-2xl shadow-destructive/30 hover:scale-[1.01] active:scale-[0.99] transition-all bg-gradient-to-r from-destructive to-destructive/80"
        >
          <RefreshCw class="w-5 h-5 mr-4" />
          Force Re-check
        </Button>
      </div>
    </div>
  </div>
</div>
