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
    Bot,
    CheckCircle2,
    Eye,
    EyeOff,
    KeyRound,
    Loader2,
    Lock,
    Server,
    ShieldCheck,
    Trash2,
    XCircle,
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  interface AiConfigPublic {
    provider: string;
    model: string;
    ollama_base_url: string | null;
    has_api_key: boolean;
    auth_method: string;
  }

  type Provider = "claude" | "openai" | "gemini" | "grok" | "ollama";
  type AuthMethod = "api_key" | "oauth_token";

  interface ProviderDef {
    id: Provider;
    label: string;
    models: string[];
    /** Authentication methods this provider supports. */
    authMethods: AuthMethod[];
    /** Human-readable note shown when OAuth is not supported. */
    noOauthNote?: string;
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
      authMethods: ["api_key"],
      noOauthNote: "Anthropic's API uses API keys only.",
    },
    {
      id: "openai",
      label: "OpenAI",
      models: ["gpt-4o-mini", "gpt-4o", "gpt-4-turbo"],
      authMethods: ["api_key"],
      noOauthNote: "OpenAI's direct API uses API keys only.",
    },
    {
      id: "gemini",
      label: "Gemini (Google)",
      models: ["gemini-2.0-flash", "gemini-1.5-pro", "gemini-1.5-flash"],
      authMethods: ["api_key", "oauth_token"],
    },
    {
      id: "grok",
      label: "Grok (xAI)",
      models: ["grok-3", "grok-3-mini", "grok-2-1212"],
      authMethods: ["api_key"],
      noOauthNote: "xAI's API uses API keys only.",
    },
    {
      id: "ollama",
      label: "Ollama (local)",
      models: ["llama3.2", "mistral", "qwen2.5-coder"],
      authMethods: [],
    },
  ];

  let provider = $state<Provider>("claude");
  let model = $state("");
  let apiKey = $state("");
  let authMethod = $state<AuthMethod>("api_key");
  let ollamaBaseUrl = $state("http://localhost:11434");
  let hasApiKey = $state(false);
  let showApiKey = $state(false);
  let apiKeyEditing = $state(false);
  let saving = $state(false);
  let clearing = $state(false);

  const selectedProvider = $derived(PROVIDERS.find((p) => p.id === provider)!);
  const supportsOAuth = $derived(
    selectedProvider.authMethods.includes("oauth_token"),
  );
  const needsCredential = $derived(provider !== "ollama");
  const isOllamaProvider = $derived(provider === "ollama");

  // When switching provider, reset auth method to first supported
  function selectProvider(id: Provider) {
    provider = id;
    model = "";
    apiKeyEditing = false;
    const def = PROVIDERS.find((p) => p.id === id)!;
    authMethod = def.authMethods[0] ?? "api_key";
  }

  async function loadConfig() {
    try {
      const cfg = await invoke<AiConfigPublic>("get_ai_config");
      if (cfg.provider) provider = cfg.provider as Provider;
      if (cfg.model) model = cfg.model;
      if (cfg.ollama_base_url) ollamaBaseUrl = cfg.ollama_base_url;
      if (cfg.auth_method) authMethod = cfg.auth_method as AuthMethod;
      hasApiKey = cfg.has_api_key;
    } catch (e) {
      console.error("[AiSettings] failed to load config:", e);
    }
  }

  async function save() {
    saving = true;
    try {
      await invoke("save_ai_config_cmd", {
        provider,
        model,
        apiKey: apiKey || null,
        ollamaBaseUrl: isOllamaProvider ? ollamaBaseUrl : null,
        authMethod,
      });
      hasApiKey =
        needsCredential && !isOllamaProvider ? apiKey.length > 0 : false;
      apiKey = "";
      apiKeyEditing = false;
      toast.success("AI settings saved");
    } catch (e: any) {
      toast.error(typeof e === "string" ? e : "Failed to save AI settings");
    } finally {
      saving = false;
    }
  }

  async function clear() {
    clearing = true;
    try {
      await invoke("save_ai_config_cmd", {
        provider: "",
        model: "",
        apiKey: null,
        ollamaBaseUrl: null,
        authMethod: "api_key",
      });
      provider = "claude";
      model = "";
      apiKey = "";
      apiKeyEditing = false;
      authMethod = "api_key";
      ollamaBaseUrl = "http://localhost:11434";
      hasApiKey = false;
      toast.success("AI settings cleared");
    } catch (e: any) {
      toast.error("Failed to clear AI settings");
    } finally {
      clearing = false;
    }
  }

  onMount(loadConfig);
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
            Configure the AI model used to generate pull change summaries.
          </CardDescription>
        </div>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
      <!-- Provider selector -->
      <div class="space-y-2">
        <Label class="text-sm font-semibold">Provider</Label>
        <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {#each PROVIDERS as p}
            {@const oauthSupported = p.authMethods.includes("oauth_token")}
            {@const noAuth = p.authMethods.length === 0}
            <button
              type="button"
              onclick={() => selectProvider(p.id)}
              class="relative px-3 py-2.5 rounded-xl text-[12px] font-semibold border transition-all duration-140 text-left
                     {provider === p.id
                ? 'bg-primary/10 text-primary border-primary/30 shadow-sm'
                : 'bg-white/5 text-muted-foreground border-white/10 hover:bg-white/10 hover:text-foreground'}"
            >
              <span class="block truncate">{p.label}</span>
              <!-- Auth badge -->
              <span class="mt-1 flex items-center gap-1">
                {#if noAuth}
                  <span
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1.5 py-0.5 rounded-full bg-slate-100 text-slate-500 border border-slate-200"
                  >
                    <Lock class="w-2.5 h-2.5" />
                    No auth
                  </span>
                {:else if oauthSupported}
                  <span
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1.5 py-0.5 rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200"
                  >
                    <ShieldCheck class="w-2.5 h-2.5" />
                    OAuth
                  </span>
                  <span
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1.5 py-0.5 rounded-full bg-blue-50 text-blue-700 border border-blue-200"
                  >
                    <KeyRound class="w-2.5 h-2.5" />
                    API key
                  </span>
                {:else}
                  <span
                    class="inline-flex items-center gap-0.5 text-[9px] font-semibold px-1.5 py-0.5 rounded-full bg-blue-50 text-blue-700 border border-blue-200"
                  >
                    <KeyRound class="w-2.5 h-2.5" />
                    API key only
                  </span>
                {/if}
              </span>
            </button>
          {/each}
        </div>
      </div>

      <!-- OAuth support info banner -->
      <div
        class="flex items-start gap-3 px-3.5 py-3 rounded-xl border text-[12px]
               {supportsOAuth
          ? 'bg-emerald-50/60 border-emerald-200/70 text-emerald-800'
          : isOllamaProvider
            ? 'bg-slate-50/60 border-slate-200/70 text-slate-600'
            : 'bg-amber-50/60 border-amber-200/70 text-amber-800'}"
      >
        {#if supportsOAuth}
          <ShieldCheck class="w-4 h-4 mt-0.5 shrink-0 text-emerald-600" />
          <span>
            <strong>OAuth 2.0 supported.</strong> You can authenticate with a Google
            OAuth access token (via Google Cloud IAM) instead of an API key. Select
            your preferred method below.
          </span>
        {:else if isOllamaProvider}
          <Lock class="w-4 h-4 mt-0.5 shrink-0 text-slate-400" />
          <span>
            <strong>No authentication required.</strong> Ollama runs locally — no
            API key or token needed.
          </span>
        {:else}
          <XCircle class="w-4 h-4 mt-0.5 shrink-0 text-amber-600" />
          <span>
            <strong>OAuth not supported.</strong>
            {selectedProvider.noOauthNote}
          </span>
        {/if}
      </div>

      <!-- Auth method selector (Gemini only) -->
      {#if supportsOAuth}
        <div class="space-y-2">
          <Label class="text-sm font-semibold">Authentication Method</Label>
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              onclick={() => (authMethod = "api_key")}
              class="flex items-center gap-2 px-3 py-2.5 rounded-xl text-[12px] font-semibold border transition-all duration-140
                     {authMethod === 'api_key'
                ? 'bg-primary/10 text-primary border-primary/30'
                : 'bg-white/5 text-muted-foreground border-white/10 hover:bg-white/10'}"
            >
              <KeyRound class="w-3.5 h-3.5 shrink-0" />
              API Key
            </button>
            <button
              type="button"
              onclick={() => (authMethod = "oauth_token")}
              class="flex items-center gap-2 px-3 py-2.5 rounded-xl text-[12px] font-semibold border transition-all duration-140
                     {authMethod === 'oauth_token'
                ? 'bg-emerald-50 text-emerald-700 border-emerald-300'
                : 'bg-white/5 text-muted-foreground border-white/10 hover:bg-white/10'}"
            >
              <ShieldCheck class="w-3.5 h-3.5 shrink-0" />
              OAuth Token
            </button>
          </div>
        </div>
      {/if}

      <!-- Model name -->
      <div class="space-y-2">
        <Label class="text-sm font-semibold">Model</Label>
        <Input
          bind:value={model}
          placeholder={selectedProvider.models[0]}
          class="bg-white/5 border-white/10 rounded-xl"
        />
        <p class="text-[10px] text-muted-foreground opacity-60">
          Common models: {selectedProvider.models.join(", ")}
        </p>
      </div>

      <!-- API key / OAuth token input -->
      {#if needsCredential && !isOllamaProvider}
        <div class="space-y-2">
          <Label class="text-sm font-semibold">
            {authMethod === "oauth_token" ? "OAuth Access Token" : "API Key"}
          </Label>
          <div class="relative">
            {#if authMethod === "oauth_token"}
              <ShieldCheck
                class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-emerald-500 opacity-60"
              />
            {:else}
              <KeyRound
                class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground opacity-40"
              />
            {/if}
            <Input
              type={showApiKey && apiKeyEditing ? "text" : "password"}
              value={hasApiKey && !apiKeyEditing ? "••••••••••••••••" : apiKey}
              onfocus={() => (apiKeyEditing = true)}
              onblur={() => {
                if (!apiKey) apiKeyEditing = false;
              }}
              oninput={(e) => {
                apiKey = (e.currentTarget as HTMLInputElement).value;
              }}
              placeholder={authMethod === "oauth_token"
                ? "Paste your Google OAuth access token"
                : "Enter your API key"}
              class="pl-10 pr-10 bg-white/5 border-white/10 rounded-xl font-mono text-[13px]"
            />
            <button
              type="button"
              onclick={() => (showApiKey = !showApiKey)}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
            >
              {#if showApiKey}
                <EyeOff class="w-4 h-4" />
              {:else}
                <Eye class="w-4 h-4" />
              {/if}
            </button>
          </div>
          {#if hasApiKey && !apiKeyEditing}
            <div class="flex items-center gap-1.5 text-[11px] text-emerald-600">
              <CheckCircle2 class="w-3 h-3" />
              {authMethod === "oauth_token" ? "OAuth token" : "API key"} saved. Click
              to replace.
            </div>
          {/if}
        </div>
      {/if}

      <!-- Ollama base URL -->
      {#if isOllamaProvider}
        <div class="space-y-2">
          <Label class="text-sm font-semibold">Ollama Base URL</Label>
          <div class="relative">
            <Server
              class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground opacity-40"
            />
            <Input
              bind:value={ollamaBaseUrl}
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
        <Button onclick={save} disabled={saving || !model} class="rounded-xl">
          {#if saving}
            <Loader2 class="w-4 h-4 mr-2 animate-spin" />
          {/if}
          Save
        </Button>
        <Button
          variant="ghost"
          onclick={clear}
          disabled={clearing}
          class="rounded-xl text-muted-foreground hover:text-destructive"
        >
          <Trash2 class="w-4 h-4 mr-1.5" />
          Clear
        </Button>
      </div>
    </CardContent>
  </Card>
</div>
