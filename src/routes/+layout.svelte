<script lang="ts">
  import '../app.css';
  import { onMount, type Snippet } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import GitMissingError from '$lib/components/custom/GitMissingError.svelte';
  import { Toaster } from "$lib/components/ui/sonner";
  import { ModeWatcher } from "mode-watcher";
  import { page } from '$app/state';

  let { children }: { children: Snippet } = $props();
  let gitInstalled = $state(true);
  let checking = $state(true);

  onMount(async () => {
    try {
      gitInstalled = await invoke('is_git_installed');
    } catch (e) {
      console.error('Failed to check git:', e);
      gitInstalled = false;
    } finally {
      checking = false;
    }
  });
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

        <nav class="flex-1 p-4 space-y-2">
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

