<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { invoke } from "@tauri-apps/api/core";
  import {
    CheckCircle2,
    RefreshCw,
    ShieldAlert,
    Terminal,
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  let gitPath = $state("");
  let version = $state<string | null>(null);
  let verifying = $state(false);

  async function loadSettings() {
    const config = await invoke<any>("get_config");
    gitPath = config.git_path;
    await verify();
  }

  async function verify() {
    verifying = true;
    try {
      version = await invoke("verify_git_path", { path: gitPath });
    } catch (e) {
      version = null;
    } finally {
      verifying = false;
    }
  }

  async function updatePath() {
    try {
      const config = await invoke<any>("get_config");
      config.git_path = gitPath;
      await invoke("set_config", { config });
      toast.success("Binary Path Updated");
      await verify();
    } catch (e) {
      toast.error("Failed to update");
    }
  }

  onMount(loadSettings);
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
  <Card class="glass border-white/5">
    <CardHeader>
      <div class="flex items-center space-x-3">
        <div>
          <CardTitle class="text-lg">Git Environment</CardTitle>
          <CardDescription
            >Configure local Git binary interaction.</CardDescription
          >
        </div>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
      <div class="space-y-4">
        <div class="space-y-2">
          <Label class="text-sm font-semibold">Executable Path</Label>
          <div class="flex gap-2">
            <div class="relative flex-1">
              <Terminal
                class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground opacity-40"
              />
              <Input
                bind:value={gitPath}
                class="pl-10 bg-white/5 border-white/10 rounded-xl"
                placeholder="git"
              />
            </div>
            <Button
              variant="secondary"
              class="rounded-xl border border-white/10"
              onclick={updatePath}
            >
              Update
            </Button>
          </div>
          <p class="text-[10px] text-muted-foreground opacity-60">
            Usually 'git' if installed in system PATH.
          </p>
        </div>

        <div
          class="p-4 rounded-2xl border border-white/5 bg-white/[0.01] flex items-center justify-between"
        >
          <div class="flex items-center gap-3">
            <div
              class="p-2 rounded-full {version
                ? 'bg-emerald-500/10 text-emerald-500'
                : 'bg-destructive/10 text-destructive'}"
            >
              {#if version}
                <CheckCircle2 class="w-4 h-4" />
              {:else}
                <ShieldAlert class="w-4 h-4" />
              {/if}
            </div>
            <div>
              <p
                class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest leading-none mb-1"
              >
                Status
              </p>
              <p
                class="text-xs font-mono {version
                  ? 'text-foreground'
                  : 'text-destructive'}"
              >
                {verifying ? "Verifying..." : version || "Binary Not Found"}
              </p>
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            class="rounded-full text-[10px] h-7"
            onclick={verify}
            disabled={verifying}
          >
            <RefreshCw class="w-3 h-3 mr-1 {verifying ? 'animate-spin' : ''}" />
            Re-verify
          </Button>
        </div>
      </div>
    </CardContent>
  </Card>
</div>
