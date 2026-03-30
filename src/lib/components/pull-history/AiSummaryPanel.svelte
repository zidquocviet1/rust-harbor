<script lang="ts">
  import ProviderIcon from "$lib/components/icons/ProviderIcon.svelte";
  import {
    type AiConfigPublic,
    type PullHistoryFile,
    fetchAiConfig,
    fetchAiConfigs,
    generatePullSummary,
  } from "$lib/stores/pullHistoryStore";
  import {
    Bot,
    ChevronDown,
    Loader2,
    RefreshCw,
    Settings,
    Sparkles,
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import type { ModelItem } from "./ModelSelectDropdown.svelte";
  import ModelSelectDropdown from "./ModelSelectDropdown.svelte";

  interface Props {
    pullId: number;
    initialSummary?: string | null;
    initialProvider?: string | null;
    initialModel?: string | null;
    files?: PullHistoryFile[];
    onFileClick?: (fileId: number) => void;
  }

  let {
    pullId,
    initialSummary = null,
    initialProvider = null,
    initialModel = null,
    files = [],
    onFileClick,
  }: Props = $props();

  let summary = $state<string | null>(null);
  let collapsed = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let aiConfig = $state<AiConfigPublic | null>(null);
  /** Model snapshot saved with this pull entry. */
  let usedModel = $state<string | null>(null);
  let usedProvider = $state<string | null>(null);
  /** All configured provider+model combos available for (re)generation. */
  let regenItems = $state<ModelItem[]>([]);
  /** Currently selected provider for regeneration. */
  let regenProvider = $state<string>("");
  /** Currently selected model for regeneration. */
  let regenModel = $state<string>("");

  const PROVIDER_LABELS: Record<string, string> = {
    claude: "Claude",
    openai: "OpenAI",
    gemini: "Gemini",
    grok: "Grok",
    ollama: "Ollama",
  };

  function renderMarkdown(text: string): string {
    return text
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/\*(.+?)\*/g, "<em>$1</em>")
      .replace(
        /`([^`]+)`/g,
        '<code class="px-1 py-0.5 rounded bg-muted font-mono text-[12px]">$1</code>',
      )
      .replace(
        /^### (.+)$/gm,
        '<h3 class="inline-flex items-center gap-1.5 text-[11px] font-black uppercase tracking-widest mt-4 mb-1.5 px-2 py-1 rounded-lg bg-primary/8 text-primary border-l-2 border-primary/40">$1</h3>',
      )
      .replace(
        /^## (.+)$/gm,
        '<h2 class="inline-flex items-center gap-1.5 text-[11.5px] font-black uppercase tracking-widest mt-4 mb-1.5 px-2.5 py-1 rounded-lg bg-primary/10 text-primary border-l-[3px] border-primary/60">$1</h2>',
      )
      .replace(/^[-•] (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
      .replace(/\n\n/g, "<br><br>")
      .replace(/\n/g, "<br>");
  }

  function injectFileLinks(
    text: string,
    changedFiles: PullHistoryFile[],
  ): string {
    if (!changedFiles.length) return renderMarkdown(text);

    const links = new Map<string, { id: number; display: string }>();
    const basenames = new Map<string, number>();

    for (const f of changedFiles) {
      const base = f.file_path.split("/").at(-1) ?? f.file_path;
      basenames.set(base, (basenames.get(base) ?? 0) + 1);
    }
    for (const f of changedFiles) {
      links.set(f.file_path, {
        id: f.id,
        display: f.file_path.split("/").at(-1) ?? f.file_path,
      });
      const base = f.file_path.split("/").at(-1) ?? f.file_path;
      if (base !== f.file_path && basenames.get(base) === 1) {
        links.set(base, { id: f.id, display: base });
      }
    }

    const sorted = [...links.entries()].sort(
      (a, b) => b[0].length - a[0].length,
    );
    const PLACEHOLDER_PREFIX = "\x00FILE\x00";
    const placeholderMap = new Map<string, { id: number; display: string }>();

    let processed = text;
    for (const [term, meta] of sorted) {
      const placeholder = `${PLACEHOLDER_PREFIX}${meta.id}\x00`;
      placeholderMap.set(placeholder, meta);
      const escaped = term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      processed = processed.replace(new RegExp(escaped, "g"), placeholder);
    }

    let html = renderMarkdown(processed);

    for (const [placeholder, { id, display }] of placeholderMap) {
      const escapedPh = placeholder.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      html = html.replace(
        new RegExp(escapedPh, "g"),
        `<button type="button" ` +
          `class="file-link inline-flex items-center gap-0.5 font-mono text-[11px] ` +
          `px-1.5 py-0.5 rounded-md bg-primary/8 text-primary border border-primary/20 ` +
          `hover:bg-primary/15 hover:border-primary/40 transition-colors cursor-pointer align-baseline mx-0.5" ` +
          `data-file-id="${id}" title="Jump to ${display}">` +
          `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" ` +
          `style="display:inline;vertical-align:middle;margin-right:2px">` +
          `<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>` +
          `<polyline points="14 2 14 8 20 8"/></svg>` +
          `${display}</button>`,
      );
    }

    return html;
  }

  const renderedHtml = $derived(
    summary !== null ? injectFileLinks(summary, files) : "",
  );

  function handleSummaryClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    const btn = target.closest(".file-link") as HTMLElement | null;
    if (btn?.dataset.fileId) {
      onFileClick?.(Number(btn.dataset.fileId));
    }
  }

  const isAiReady = $derived(
    aiConfig !== null &&
      aiConfig.provider !== "" &&
      (aiConfig.has_api_key || aiConfig.provider === "ollama"),
  );

  const activeProvider = $derived(aiConfig?.provider ?? "");
  const configuredModel = $derived(aiConfig?.model ?? "");

  async function loadConfig() {
    try {
      // Load active provider config (for the initial generate state)
      aiConfig = await fetchAiConfig();

      // Load ALL configured providers to build the regen items list
      const all = await fetchAiConfigs();
      const items: ModelItem[] = [];
      for (const [provId, pc] of Object.entries(all.providers)) {
        if (pc.model) {
          items.push({ provider: provId, model: pc.model });
        }
      }
      regenItems = items;

      // Default regen selection to the active provider
      if (!regenProvider && all.active_provider) {
        regenProvider = all.active_provider;
        const activePc = all.providers[all.active_provider];
        regenModel = activePc?.model ?? "";
      }
    } catch {
      aiConfig = {
        provider: "",
        model: "",
        ollama_base_url: null,
        has_api_key: false,
        auth_method: "api_key",
      };
    }
  }

  async function generate(
    forceRegenerate = false,
    provider?: string,
    model?: string,
  ) {
    loading = true;
    error = null;
    summary = null;

    const useProvider = provider ?? activeProvider;
    const useModel = model ?? configuredModel;
    let nextSummary: string | null = null;
    let nextError: string | null = null;

    try {
      const result = await generatePullSummary(
        pullId,
        forceRegenerate,
        useModel || undefined,
        useProvider || undefined,
      );
      nextSummary = result ?? null;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message ?? "");
      nextError = msg || "Failed to generate summary. Please try again.";
    }

    summary = nextSummary;
    error = nextError;
    if (nextSummary !== null) {
      usedModel = useModel || null;
      usedProvider = useProvider || null;
    }
    loading = false;
  }

  onMount(() => {
    summary = initialSummary || null;
    usedModel = initialModel || null;
    usedProvider = initialProvider || null;
    loadConfig();
  });
</script>

<div class="bg-white/80 rounded-2xl border border-slate-200/70 overflow-hidden">
  <!-- Header -->
  <div
    class="px-4 py-2.5 flex items-center gap-2 {!collapsed
      ? 'border-b border-slate-100'
      : ''}"
  >
    {#if summary !== null && !loading}
      <button
        type="button"
        onclick={() => (collapsed = !collapsed)}
        class="flex items-center gap-1.5 text-[11px] font-black uppercase tracking-widest
               text-muted-foreground hover:text-foreground transition-colors shrink-0"
      >
        <Bot size={12} strokeWidth={2.5} />
        AI Summary
        <ChevronDown
          size={13}
          strokeWidth={2.5}
          class="transition-transform duration-200"
          style="transform: {collapsed ? 'rotate(-90deg)' : 'rotate(0deg)'};"
        />
      </button>

      <!-- Regenerate controls: model picker across all configured providers + button -->
      <div class="flex items-center gap-2 ml-auto shrink-0">
        {#if regenItems.length > 0}
          <div class="w-[200px]">
            <ModelSelectDropdown
              items={regenItems}
              value={regenModel || configuredModel}
              valueProvider={regenProvider || activeProvider}
              onchange={(p, m) => {
                regenProvider = p;
                regenModel = m;
              }}
            />
          </div>
        {/if}
        <button
          type="button"
          onclick={() =>
            generate(
              true,
              regenProvider || activeProvider,
              regenModel || configuredModel,
            )}
          class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-primary transition-colors shrink-0"
        >
          <RefreshCw size={11} strokeWidth={2.5} />
          Regenerate
        </button>
      </div>
    {:else}
      <span
        class="flex items-center gap-1.5 text-[11px] font-black uppercase tracking-widest text-muted-foreground"
      >
        <Bot size={12} strokeWidth={2.5} />
        AI Summary
      </span>
    {/if}
  </div>

  <!-- Body -->
  {#if !collapsed}
    <div class="p-4">
      {#if loading}
        <div class="flex items-center gap-2 text-[13px] text-muted-foreground">
          <Loader2 size={14} class="animate-spin" />
          Generating summary…
        </div>
      {:else if error}
        <div class="space-y-3">
          <p class="text-[12px] text-destructive">{error}</p>
          {#if isAiReady}
            <div class="flex flex-col gap-2">
              {#if activeProvider}
                <div
                  class="flex items-center gap-2 px-3 py-2 rounded-xl bg-slate-50 border border-slate-100 w-fit"
                >
                  <ProviderIcon provider={activeProvider} />
                  <span class="text-[12px] font-semibold text-foreground"
                    >{PROVIDER_LABELS[activeProvider] ?? activeProvider}</span
                  >
                  {#if configuredModel}
                    <span class="w-1 h-1 rounded-full bg-border shrink-0"
                    ></span>
                    <span class="text-[11px] text-muted-foreground font-mono"
                      >{configuredModel}</span
                    >
                  {/if}
                </div>
              {/if}
              <button
                type="button"
                onclick={() => generate(false)}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-[12px] font-semibold
                     bg-primary/10 text-primary hover:bg-primary/15 transition-colors border border-primary/20 w-fit"
              >
                <Sparkles size={12} strokeWidth={2.5} />
                Try again
              </button>
            </div>
          {/if}
        </div>
      {:else if summary !== null}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="text-[13px] text-foreground leading-relaxed prose-sm max-w-none"
          onclick={handleSummaryClick}
        >
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html renderedHtml}
        </div>

        <!-- Model attribution hint -->
        {#if usedProvider || usedModel}
          <div
            class="flex items-center gap-1.5 mt-3 pt-3 border-t border-slate-100"
          >
            <span
              class="text-[10px] text-muted-foreground/50 uppercase tracking-widest font-semibold"
              >Generated with</span
            >
            {#if usedProvider}
              <ProviderIcon provider={usedProvider} />
            {/if}
            {#if usedModel}
              <span
                class="text-[11px] text-muted-foreground/70 font-mono"
                title={usedModel}>{usedModel}</span
              >
            {/if}
          </div>
        {/if}
      {:else if !isAiReady}
        <div class="flex items-center gap-3 py-1">
          <div class="p-2 rounded-xl bg-muted text-muted-foreground shrink-0">
            <Settings size={14} strokeWidth={2} />
          </div>
          <div class="space-y-0.5">
            <p class="text-[12px] font-semibold text-foreground">
              AI not configured
            </p>
            <p class="text-[11px] text-muted-foreground">
              Set up an AI provider in
              <a
                href="/settings"
                class="text-primary underline underline-offset-2 hover:text-primary/80"
              >
                Settings → AI Settings
              </a>
              to generate summaries.
            </p>
          </div>
        </div>
      {:else}
        <!-- Active provider info chip + generate button -->
        <div class="flex flex-col gap-3">
          {#if activeProvider}
            <div
              class="flex items-center gap-2 px-3 py-2 rounded-xl bg-slate-50 border border-slate-100 w-fit"
            >
              <ProviderIcon provider={activeProvider} />
              <span class="text-[12px] font-semibold text-foreground"
                >{PROVIDER_LABELS[activeProvider] ?? activeProvider}</span
              >
              {#if configuredModel}
                <span class="w-1 h-1 rounded-full bg-border shrink-0"></span>
                <span class="text-[11px] text-muted-foreground font-mono"
                  >{configuredModel}</span
                >
              {/if}
            </div>
          {/if}
          <button
            type="button"
            onclick={() => generate(false)}
            class="inline-flex items-center gap-2 px-4 py-2 rounded-xl text-[13px] font-semibold
                 bg-primary/10 text-primary hover:bg-primary/15 transition-all duration-140
                 border border-primary/20 hover:border-primary/30 hover:shadow-sm w-fit"
          >
            <Sparkles size={14} strokeWidth={2} />
            Summarize with AI
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
