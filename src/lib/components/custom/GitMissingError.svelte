<script lang="ts">
  import { AlertCircle, Terminal, Download, RefreshCw } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
</script>

<div class="fixed inset-0 z-[200] flex items-center justify-center p-6 overflow-hidden">
  <!-- Backdrop Blur -->
  <div class="absolute inset-0 bg-background/40 backdrop-blur-2xl"></div>
  
  <!-- Ambient Error Glow -->
  <div class="absolute w-[500px] h-[500px] bg-destructive/10 rounded-full blur-[120px] animate-pulse"></div>

  <div class="relative max-w-xl w-full">
    <div class="glass border-destructive/20 rounded-[2rem] p-10 space-y-8 shadow-[0_0_50px_-12px_rgba(239,68,68,0.2)] animate-in fade-in zoom-in duration-500">
      <div class="flex flex-col items-center text-center space-y-6">
        <div class="relative group">
          <div class="absolute inset-0 bg-destructive/30 rounded-3xl blur-xl group-hover:blur-2xl transition-all duration-500"></div>
          <div class="relative p-6 bg-destructive/10 border border-destructive/20 rounded-3xl text-destructive shadow-inner">
            <AlertCircle class="w-16 h-16" />
          </div>
        </div>
        
        <div class="space-y-2">
          <h1 class="text-4xl font-black tracking-tighter text-glow text-destructive">Git Not Detected</h1>
          <p class="text-muted-foreground text-lg leading-relaxed max-w-sm mx-auto font-medium">
            Harbor requires the <span class="text-foreground border-b border-muted-foreground/30">Git CLI</span> to communicate with your repositories.
          </p>
        </div>
      </div>
      
      <div class="grid grid-cols-1 gap-4">
        <div class="flex items-start p-4 rounded-2xl bg-white/5 border border-white/5 space-x-4">
          <div class="p-2 rounded-lg bg-primary/10 text-primary">
            <Terminal class="w-5 h-5" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-bold">Inaccessible Binary</p>
            <p class="text-xs text-muted-foreground">The 'git' command was not found in your system's PATH environment variable.</p>
          </div>
        </div>

        <div class="flex items-start p-4 rounded-2xl bg-white/5 border border-white/5 space-x-4">
          <div class="p-2 rounded-lg bg-primary/10 text-primary">
            <Download class="w-5 h-5" />
          </div>
          <div class="space-y-1">
            <p class="text-sm font-bold">Fast Installation</p>
            <p class="text-xs text-muted-foreground">Download and install Git from <a href="https://git-scm.com" target="_blank" class="text-primary hover:underline underline-offset-4 font-bold">git-scm.com</a>.</p>
          </div>
        </div>
      </div>
      
      <div class="pt-4">
        <Button 
          variant="destructive"
          size="lg"
          onclick={() => window.location.reload()}
          class="w-full py-7 rounded-2xl text-lg font-bold shadow-xl shadow-destructive/20 hover:scale-[1.02] active:scale-[0.98] transition-all"
        >
          <RefreshCw class="w-5 h-5 mr-3" />
          Retry Connection
        </Button>
      </div>
    </div>
  </div>
</div>
