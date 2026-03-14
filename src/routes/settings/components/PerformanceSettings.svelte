<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { toast } from "svelte-sonner";
  import { Save, Zap, Hash, X, PlusCircle, ShieldCheck } from "lucide-svelte";
  import { APP_BRANDING } from "$lib/config/branding";

  let config = $state<any>(null);
  let newPattern = $state("");

  const SYSTEM_PATTERNS = [
    "**/node_modules/**",
    "**/target/**",
    "**/.venv/**",
    "**/.git/**"
  ];

  async function loadSettings() {
    config = await invoke("get_config");
  }

  async function saveSettings() {
    try {
      await invoke("set_config", { config });
      toast.success("Settings Saved");
    } catch (e) {
      toast.error("Failed to save");
    }
  }

  function addPattern() {
    if (newPattern && !config.exclusion_patterns.includes(newPattern)) {
      config.exclusion_patterns = [...config.exclusion_patterns, newPattern];
      newPattern = "";
    }
  }

  function removePattern(pattern: string) {
    if (SYSTEM_PATTERNS.includes(pattern)) return;
    config.exclusion_patterns = config.exclusion_patterns.filter((p: string) => p !== pattern);
  }

  onMount(loadSettings);
</script>

{#if config}
  <div class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <Card class="glass border-white/5">
      <CardContent class="p-8 space-y-8">
        <div class="px-4 flex items-center justify-between">
          <div class="space-y-1">
            <CardTitle class="text-xl font-bold">Scan Optimization</CardTitle>
            <CardDescription class="text-sm">Configure how {APP_BRANDING.shortName} traverses your filesystem.</CardDescription>
          </div>
          
          <div class="flex items-center gap-4 px-4 py-2 bg-white/[0.02] border border-white/5 rounded-2xl">
            <div class="flex items-center gap-1.5">
              <div class="w-2 h-2 rounded-full bg-primary/40"></div>
              <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/60">System</span>
            </div>
            <div class="flex items-center gap-1.5">
              <div class="w-2 h-2 rounded-full bg-white/20 border border-white/10"></div>
              <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/60">User</span>
            </div>
          </div>
        </div>

        <div class="space-y-4">
          <div class="flex items-center justify-between p-4 rounded-2xl bg-white/[0.02] border border-white/5">
            <div class="space-y-0.5">
               <Label class="text-sm font-semibold">Recursion Depth</Label>
               <p class="text-[10px] text-muted-foreground opacity-60">Maximum directory nesting level for repository discovery.</p>
            </div>
            <div class="flex items-center gap-4">
              <input 
                type="range" 
                min="1" 
                max="20" 
                bind:value={config.max_depth}
                class="w-32 accent-primary"
              />
              <div class="flex items-center justify-center w-10 h-10 rounded-xl bg-primary/10 border border-primary/20 text-primary font-mono text-sm font-bold">
                {config.max_depth}
              </div>
            </div>
          </div>

          <div class="space-y-4 p-4 rounded-2xl bg-white/[0.02] border border-white/5">
            <div class="space-y-0.5">
               <Label class="text-sm font-semibold">Exclusion Patterns</Label>
               <p class="text-[10px] text-muted-foreground opacity-60">Glob patterns of heavy or irrelevant directories to ignore.</p>
            </div>
            
            <div class="flex gap-2">
              <div class="relative flex-1">
                <Input 
                  bind:value={newPattern} 
                  placeholder="e.g. **/temp/**" 
                  class="bg-white/5 border-white/10 rounded-xl h-11 pl-4"
                  onkeypress={(e) => e.key === 'Enter' && addPattern()}
                />
              </div>
              <Button size="icon" class="rounded-xl h-11 w-11 bg-primary hover:scale-105 transition-transform border border-primary/20 shadow-lg shadow-primary/10" onclick={addPattern}>
                <PlusCircle class="w-5 h-5" />
              </Button>
            </div>

            <div class="flex flex-wrap gap-2 pt-2">
               {#each config.exclusion_patterns as pattern}
                 {@const isSystem = SYSTEM_PATTERNS.includes(pattern)}
                 <div class="flex items-center gap-2 px-3 py-2 rounded-xl transition-all duration-300 {isSystem ? 'bg-white/[0.03] border-white/5 opacity-60' : 'bg-primary/5 border-primary/20 text-primary'} border text-[11px] font-mono group">
                   {#if isSystem}
                     <ShieldCheck class="w-3 h-3 text-muted-foreground/40" />
                   {:else}
                     <Hash class="w-3 h-3 text-primary/50" />
                   {/if}
                   
                   <span class={isSystem ? 'text-muted-foreground' : 'text-primary font-medium'}>{pattern}</span>
                   
                   {#if !isSystem}
                     <button 
                        class="ml-1 p-1 rounded-md hover:bg-destructive/10 hover:text-destructive transition-all opacity-0 group-hover:opacity-100" 
                        onclick={() => removePattern(pattern)}
                        aria-label="Remove pattern"
                      >
                       <X class="w-3 h-3" />
                     </button>
                   {/if}
                 </div>
               {/each}
            </div>
          </div>
        </div>

        <div class="px-4">
          <Button onclick={saveSettings} class="w-full py-6 rounded-2xl bg-primary text-primary-foreground hover:scale-[1.01] active:scale-[0.99] transition-all shadow-xl shadow-primary/20 font-bold">
            <Save class="w-4 h-4 mr-2" />
            Apply Performance Profile
          </Button>
        </div>
      </CardContent>
    </Card>
  </div>
{/if}
