<script lang="ts">
  import {
    type AiConfigPublic,
    type PullHistoryFile,
    fetchAiConfig,
    generatePullSummary,
    PROVIDER_MODELS,
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

  interface Props {
    pullId: number;
    initialSummary?: string | null;
    initialProvider?: string | null;
    initialModel?: string | null;
    /** Changed files for this pull — used to inject clickable file links into the summary. */
    files?: PullHistoryFile[];
    /** Called when the user clicks a file link in the summary. */
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
  let usedModel = $state<string | null>(null);
  let selectedModel = $state<string>("");

  // Simple markdown → HTML renderer for the subset we care about.
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

  /**
   * Inject clickable file-link buttons for every file path (or basename) mentioned
   * in the summary text. Uses a placeholder strategy so markdown rendering doesn't
   * interfere with injected HTML.
   */
  function injectFileLinks(
    text: string,
    changedFiles: PullHistoryFile[],
  ): string {
    if (!changedFiles.length) return renderMarkdown(text);

    // Build a map: searchTerm → { id, displayName }
    // Prefer full path match; also register basename if unique.
    const links = new Map<string, { id: number; display: string }>();
    const basenames = new Map<string, number>(); // basename → count across files

    for (const f of changedFiles) {
      const base = f.file_path.split("/").at(-1) ?? f.file_path;
      basenames.set(base, (basenames.get(base) ?? 0) + 1);
    }

    for (const f of changedFiles) {
      // Always register the full path
      links.set(f.file_path, {
        id: f.id,
        display: f.file_path.split("/").at(-1) ?? f.file_path,
      });
      // Register basename only if it's unique among changed files
      const base = f.file_path.split("/").at(-1) ?? f.file_path;
      if (base !== f.file_path && basenames.get(base) === 1) {
        links.set(base, { id: f.id, display: base });
      }
    }

    // Sort by length descending so longer paths are replaced before their substrings
    const sorted = [...links.entries()].sort(
      (a, b) => b[0].length - a[0].length,
    );

    // Replace with placeholders first, then run markdown, then swap in HTML
    const PLACEHOLDER_PREFIX = "\x00FILE\x00";
    const placeholderMap = new Map<string, { id: number; display: string }>();

    let processed = text;
    for (const [term, meta] of sorted) {
      const placeholder = `${PLACEHOLDER_PREFIX}${meta.id}\x00`;
      placeholderMap.set(placeholder, meta);
      const escaped = term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      processed = processed.replace(new RegExp(escaped, "g"), placeholder);
    }

    // Apply markdown to the placeholder-substituted text
    let html = renderMarkdown(processed);

    // Replace each placeholder with a styled, clickable button
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

  // Derived rendered HTML — updates reactively when summary or files change.
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

  const availableModels = $derived(
    aiConfig
      ? (PROVIDER_MODELS[aiConfig.provider] ??
          (aiConfig.model ? [aiConfig.model] : []))
      : [],
  );

  async function loadConfig() {
    try {
      aiConfig = await fetchAiConfig();
      if (!selectedModel && aiConfig.model) {
        selectedModel = aiConfig.model;
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

  async function generate(forceRegenerate = false) {
    loading = true;
    error = null;
    summary = null;

    let nextSummary: string | null = null;
    let nextError: string | null = null;
    const modelUsed = selectedModel || aiConfig?.model || null;

    try {
      const result = await generatePullSummary(
        pullId,
        forceRegenerate,
        selectedModel || undefined,
      );
      nextSummary = result ?? null;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message ?? "");
      nextError = msg || "Failed to generate summary. Please try again.";
    }

    // Apply all state updates in one synchronous block so Svelte batches them.
    summary = nextSummary;
    error = nextError;
    if (nextSummary !== null) usedModel = modelUsed;
    loading = false;
  }

  onMount(() => {
    summary = initialSummary || null;
    usedModel = initialModel || null;
    loadConfig();
  });
</script>

<div class="bg-white/80 rounded-2xl border border-slate-200/70 overflow-hidden">
  <!-- Header -->
  <div
    class="px-4 py-2.5 flex items-center justify-between {!collapsed
      ? 'border-b border-slate-100'
      : ''}"
  >
    {#if summary !== null && !loading}
      <!-- Clickable label toggles collapse -->
      <button
        type="button"
        onclick={() => (collapsed = !collapsed)}
        class="flex items-center gap-1.5 text-[11px] font-black uppercase tracking-widest
               text-muted-foreground hover:text-foreground transition-colors"
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
      <div class="flex items-center gap-2 ml-auto shrink-0">
        {#if usedModel}
          <span
            class="text-[10px] text-muted-foreground/50 font-mono truncate max-w-[110px]"
            title={usedModel}
          >
            {usedModel}
          </span>
          <span class="w-px h-3 bg-border shrink-0"></span>
        {/if}
        {#if availableModels.length > 0}
          <select
            bind:value={selectedModel}
            class="text-[11px] font-mono bg-muted/50 border border-border rounded-lg px-1.5 py-0.5 text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary/40 max-w-[130px]"
          >
            {#each availableModels as m}
              <option value={m}>{m}</option>
            {/each}
            {#if selectedModel && !availableModels.includes(selectedModel)}
              <option value={selectedModel}>{selectedModel}</option>
            {/if}
          </select>
        {/if}
        <button
          type="button"
          onclick={() => generate(true)}
          class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-primary transition-colors"
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
        <!-- Generating state -->
        <div class="flex items-center gap-2 text-[13px] text-muted-foreground">
          <Loader2 size={14} class="animate-spin" />
          Generating summary…
        </div>
      {:else if error}
        <!-- Error state -->
        <div class="space-y-3">
          <p class="text-[12px] text-destructive">{error}</p>
          {#if isAiReady}
            <div class="flex flex-col gap-2">
              {#if availableModels.length > 0}
                <div class="flex items-center gap-2">
                  <label
                    class="text-[11px] font-semibold text-muted-foreground shrink-0"
                    >Model</label
                  >
                  <select
                    bind:value={selectedModel}
                    class="flex-1 text-[12px] font-mono bg-muted/50 border border-border rounded-lg px-2 py-1 text-foreground focus:outline-none focus:ring-1 focus:ring-primary/40"
                  >
                    {#each availableModels as m}
                      <option value={m}>{m}</option>
                    {/each}
                    {#if selectedModel && !availableModels.includes(selectedModel)}
                      <option value={selectedModel}
                        >{selectedModel} (custom)</option
                      >
                    {/if}
                  </select>
                </div>
              {/if}
              <button
                type="button"
                onclick={() => generate(false)}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-[12px] font-semibold
                     bg-primary/10 text-primary hover:bg-primary/15 transition-colors border border-primary/20"
              >
                <Sparkles size={12} strokeWidth={2.5} />
                Try again
              </button>
            </div>
          {/if}
        </div>
      {:else if summary !== null}
        <!-- Summary result — onclick is event delegation for injected file-link buttons -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="text-[13px] text-foreground leading-relaxed prose-sm max-w-none"
          onclick={handleSummaryClick}
        >
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html renderedHtml}
        </div>
      {:else if !isAiReady}
        <!-- Not configured -->
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
        <!-- Model selector + generate button -->
        <div class="flex flex-col gap-3">
          {#if availableModels.length > 0 && isAiReady}
            <div class="flex items-center gap-2">
              <label
                class="text-[11px] font-semibold text-muted-foreground shrink-0"
                >Model</label
              >
              <select
                bind:value={selectedModel}
                class="flex-1 text-[12px] font-mono bg-muted/50 border border-border rounded-lg px-2 py-1 text-foreground focus:outline-none focus:ring-1 focus:ring-primary/40"
              >
                {#each availableModels as m}
                  <option value={m}>{m}</option>
                {/each}
                {#if selectedModel && !availableModels.includes(selectedModel)}
                  <option value={selectedModel}>{selectedModel} (custom)</option
                  >
                {/if}
              </select>
            </div>
          {/if}
          <button
            type="button"
            onclick={() => generate(false)}
            class="inline-flex items-center gap-2 px-4 py-2 rounded-xl text-[13px] font-semibold
                 bg-primary/10 text-primary hover:bg-primary/15 transition-all duration-140
                 border border-primary/20 hover:border-primary/30 hover:shadow-sm"
          >
            <Sparkles size={14} strokeWidth={2} />
            Summarize with AI
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
