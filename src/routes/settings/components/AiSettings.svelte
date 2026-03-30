<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import ProviderIcon from "$lib/components/icons/ProviderIcon.svelte";
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
    Bot,
    CheckCircle2,
    Eye,
    EyeOff,
    KeyRound,
    Loader2,
    Lock,
    Server,
    Trash2,
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  type Provider = "claude" | "openai" | "gemini" | "grok" | "ollama";

  interface ProviderDef {
    id: Provider;
    label: string;
    models: string[];
    requiresKey: boolean;
  }

  interface ProviderState {
    model: string;
    apiKey: string;
    ollamaBaseUrl: string;
    hasApiKey: boolean;
    apiKeyEditing: boolean;
    showApiKey: boolean;
    saving: boolean;
    clearing: boolean;
  }

  const PROVIDERS: ProviderDef[] = [
    {
      id: "claude",
      label: "Claude (Anthropic)",
      models: [
        "claude-haiku-4-5-20251001",
        "claude-sonnet-4-6",
        "claude-opus-4-6",
      ],
      requiresKey: true,
    },
    {
      id: "openai",
      label: "OpenAI",
      models: ["gpt-4o-mini", "gpt-4o", "gpt-4-turbo"],
      requiresKey: true,
    },
    {
      id: "gemini",
      label: "Gemini (Google)",
      models: ["gemini-2.0-flash", "gemini-1.5-pro", "gemini-1.5-flash"],
      requiresKey: true,
    },
    {
      id: "grok",
      label: "Grok (xAI)",
      models: ["grok-3", "grok-3-mini", "grok-2-1212"],
      requiresKey: true,
    },
    {
      id: "ollama",
      label: "Ollama (local)",
      models: ["llama3.2", "mistral", "qwen2.5-coder"],
      requiresKey: false,
    },
  ];

  function defaultState(): ProviderState {
    return {
      model: "",
      apiKey: "",
      ollamaBaseUrl: "http://localhost:11434",
      hasApiKey: false,
      apiKeyEditing: false,
      showApiKey: false,
      saving: false,
      clearing: false,
    };
  }

  let activeProvider = $state<Provider>("claude");
  let selectedTab = $state<Provider>("claude");

  let states = $state<Record<Provider, ProviderState>>({
    claude: defaultState(),
    openai: defaultState(),
    gemini: defaultState(),
    grok: defaultState(),
    ollama: defaultState(),
  });

  const tab = $derived(states[selectedTab]);
  const tabDef = $derived(PROVIDERS.find((p) => p.id === selectedTab)!);
  const isOllama = $derived(selectedTab === "ollama");

  const isConfigured = (id: Provider) =>
    states[id].model.length > 0 && (states[id].hasApiKey || id === "ollama");

  async function loadConfigs() {
    try {
      const data = await invoke<{
        active_provider: string;
        providers: Record<
          string,
          {
            model: string;
            api_key: string | null;
            has_api_key: boolean;
            ollama_base_url: string | null;
          }
        >;
      }>("get_ai_configs");

      activeProvider = (data.active_provider as Provider) || "claude";
      selectedTab = activeProvider;

      for (const [id, pc] of Object.entries(data.providers)) {
        const pid = id as Provider;
        if (states[pid]) {
          states[pid].model = pc.model;
          states[pid].apiKey = pc.api_key ?? "";
          states[pid].hasApiKey = pc.has_api_key;
          states[pid].ollamaBaseUrl =
            pc.ollama_base_url ?? "http://localhost:11434";
        }
      }
    } catch (e) {
      console.error("[AiSettings] failed to load configs:", e);
    }
  }

  async function save() {
    states[selectedTab].saving = true;
    try {
      await invoke("save_provider_config", {
        provider: selectedTab,
        model: tab.model,
        apiKey: tab.apiKey || null,
        ollamaBaseUrl: isOllama ? tab.ollamaBaseUrl : null,
        authMethod: "api_key",
      });
      activeProvider = selectedTab;
      if (tabDef.requiresKey) {
        states[selectedTab].hasApiKey = tab.hasApiKey || tab.apiKey.length > 0;
      }
      states[selectedTab].apiKey = "";
      states[selectedTab].apiKeyEditing = false;
      states[selectedTab].showApiKey = false;
      toast.success("AI settings saved");
    } catch (e: any) {
      toast.error(typeof e === "string" ? e : "Failed to save AI settings");
    } finally {
      states[selectedTab].saving = false;
    }
  }

  async function clearProvider() {
    states[selectedTab].clearing = true;
    try {
      await invoke("clear_provider_config", { provider: selectedTab });
      states[selectedTab] = defaultState();
      const data = await invoke<{ active_provider: string }>("get_ai_configs");
      activeProvider = (data.active_provider as Provider) || "claude";
      toast.success("Provider config cleared");
    } catch {
      toast.error("Failed to clear provider config");
    } finally {
      states[selectedTab].clearing = false;
    }
  }

  onMount(loadConfigs);
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
  <Card class="glass border-white/5">
    <CardHeader>
      <div class="flex items-center space-x-3">
        <div class="p-2 rounded-xl bg-primary/10 text-primary">
          <Bot class="w-5 h-5" />
        </div>
        <div>
          <CardTitle class="text-lg">AI Provider</CardTitle>
          <CardDescription>
            Configure AI providers used to generate pull change summaries.
          </CardDescription>
        </div>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
      <!-- Provider tab selector -->
      <div class="space-y-2">
        <Label class="text-sm font-semibold">Provider</Label>
        <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {#each PROVIDERS as p}
            {@const configured = isConfigured(p.id)}
            <button
              type="button"
              onclick={() => (selectedTab = p.id)}
              class="relative px-3 py-2.5 rounded-xl text-[12px] font-semibold border transition-all duration-140 text-left
                     {selectedTab === p.id
                ? 'bg-primary/10 text-primary border-primary/30 shadow-sm'
                : 'bg-white/5 text-muted-foreground border-white/10 hover:bg-white/10 hover:text-foreground'}"
            >
              <span class="flex items-center gap-2 mb-1 pr-2">
                <ProviderIcon provider={p.id} />
                <span class="truncate">{p.label}</span>
              </span>

              <span class="flex items-center gap-1 flex-wrap">
                {#if configured}
                  <span
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1.5 py-0.5 rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200"
                  >
                    <CheckCircle2 class="w-2.5 h-2.5" />
                    Configured
                  </span>
                {/if}
                {#if p.requiresKey}
                  <span
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1.5 py-0.5 rounded-full bg-blue-50 text-blue-700 border border-blue-200"
                  >
                    <KeyRound class="w-2.5 h-2.5" />
                    API key
                  </span>
                {:else}
                  <span
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1.5 py-0.5 rounded-full bg-slate-100 text-slate-500 border border-slate-200"
                  >
                    <Lock class="w-2.5 h-2.5" />
                    No auth
                  </span>
                {/if}
              </span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Auth info banner -->
      <div
        class="flex items-start gap-3 px-3.5 py-3 rounded-xl border text-[12px]
                  {isOllama
          ? 'bg-slate-50/60 border-slate-200/70 text-slate-600'
          : 'bg-blue-50/60 border-blue-200/70 text-blue-800'}"
      >
        {#if isOllama}
          <Lock class="w-4 h-4 mt-0.5 shrink-0 text-slate-400" />
          <span
            ><strong>No authentication required.</strong> Ollama runs locally — no
            API key needed.</span
          >
        {:else}
          <KeyRound class="w-4 h-4 mt-0.5 shrink-0 text-blue-500" />
          <span
            ><strong>API key required.</strong> Your key is stored locally and
            never sent anywhere except the {tabDef.label} API.</span
          >
        {/if}
      </div>

      <!-- Model name -->
      <div class="space-y-2">
        <Label class="text-sm font-semibold">Model</Label>
        <Input
          bind:value={states[selectedTab].model}
          placeholder={tabDef.models[0]}
          class="bg-white/5 border-white/10 rounded-xl"
        />
        <p class="text-[10px] text-muted-foreground opacity-60">
          Common models: {tabDef.models.join(", ")}
        </p>
      </div>

      <!-- API key input -->
      {#if tabDef.requiresKey}
        <div class="space-y-2">
          <Label class="text-sm font-semibold">API Key</Label>
          <div class="relative">
            <KeyRound
              class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground opacity-40"
            />
            <Input
              type={tab.showApiKey ? "text" : "password"}
              value={tab.apiKey}
              onfocus={() => (states[selectedTab].apiKeyEditing = true)}
              onblur={() => {
                if (!tab.apiKey) {
                  states[selectedTab].apiKeyEditing = false;
                  states[selectedTab].showApiKey = false;
                }
              }}
              oninput={(e) => {
                states[selectedTab].apiKey = (
                  e.currentTarget as HTMLInputElement
                ).value;
              }}
              placeholder="Enter your API key"
              class="pl-10 pr-10 bg-white/5 border-white/10 rounded-xl font-mono text-[13px]"
            />
            {#if tab.hasApiKey || tab.apiKey}
              <button
                type="button"
                onclick={() =>
                  (states[selectedTab].showApiKey = !tab.showApiKey)}
                class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              >
                {#if tab.showApiKey}
                  <Eye class="w-4 h-4" />
                {:else}
                  <EyeOff class="w-4 h-4" />
                {/if}
              </button>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Ollama base URL -->
      {#if isOllama}
        <div class="space-y-2">
          <Label class="text-sm font-semibold">Ollama Base URL</Label>
          <div class="relative">
            <Server
              class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground opacity-40"
            />
            <Input
              bind:value={states[selectedTab].ollamaBaseUrl}
              placeholder="http://localhost:11434"
              class="pl-10 bg-white/5 border-white/10 rounded-xl font-mono text-[13px]"
            />
          </div>
          <p class="text-[10px] text-muted-foreground opacity-60">
            Ensure your Ollama server is running before generating summaries.
          </p>
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-2 pt-2">
        <Button
          onclick={save}
          disabled={tab.saving || !tab.model}
          class="rounded-xl"
        >
          {#if tab.saving}
            <Loader2 class="w-4 h-4 mr-2 animate-spin" />
          {/if}
          Save
        </Button>
        <Button
          variant="ghost"
          onclick={clearProvider}
          disabled={tab.clearing}
          class="rounded-xl text-muted-foreground hover:text-destructive"
        >
          <Trash2 class="w-4 h-4 mr-1.5" />
          Clear
        </Button>
      </div>
    </CardContent>
  </Card>
</div>
