<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { toast } from "svelte-sonner";
  import Fuse from "fuse.js";
  import { 
    RefreshCw, 
    GitBranch, 
    FolderOpen, 
    ExternalLink,
    ArrowDown,
    ArrowUp,
    Globe,
    AlertTriangle,
    Search,
    Terminal,
    Clock,
    Code2,
    LayoutGrid,
    LayoutList,
    CheckCircle2,
    GitCommit,
    ArrowRightLeft,
    Check,
    X,
    FileText,
    Eye,
    Tag
  } from "lucide-svelte";
  import { tick } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import hljs from 'highlight.js';
  import 'highlight.js/styles/github-dark.css';
  import GroupHeader from "$lib/components/custom/GroupHeader.svelte";
  import OpenInEditor from "$lib/components/custom/OpenInEditor.svelte";
  import * as si from "simple-icons";
  import type { SimpleIcon } from "simple-icons";
  import {
    siC,
    siClojure,
    siCplusplus,
    siCss,
    siDart,
    siDocker,
    siDotnet,
    siElixir,
    siFsharp,
    siGo,
    siHaskell,
    siHtml5,
    siJavascript,
    siJson,
    siKotlin,
    siLua,
    siMarkdown,
    siNixos,
    siOpenjdk,
    siPerl,
    siPhp,
    siPython,
    siR,
    siReact,
    siRuby,
    siRust,
    siScala,
    siShell,
    siSvelte,
    siSwift,
    siToml,
    siTypescript,
    siVuedotjs,
    siYaml,
    siZig
  } from "simple-icons";
  import { 
    allTags, 
    tagLoading,
    selectedTagIds,
    activeTagFilters,
    assignTag,
    removeTag,
    loadTags,
    type Tag as StoreTag
  } from "$lib/stores/tagStore";

  type SyncStatus = 'Clean' | 'Ahead' | 'Dirty' | 'Behind' | 'Diverged';

  interface RepoMetadata {
    name: string;
    path: string;
    description: string | null;
    branch: string;
    sync_status: SyncStatus;
    remote_url: string | null;
    remote_reachable: boolean;
    last_modified: number;
    languages: Record<string, number>;
    tags: string[];
  }

  const LANGUAGE_ICON_MAP: Record<string, SimpleIcon> = {
    "rust": siRust,
    "typescript": siTypescript,
    "javascript": siJavascript,
    "python": siPython,
    "go": siGo,
    "c": siC,
    "c++": siCplusplus,
    "c#": siDotnet,
    "f#": siFsharp,
    "java": siOpenjdk,
    "kotlin": siKotlin,
    "swift": siSwift,
    "dart": siDart,
    "php": siPhp,
    "ruby": siRuby,
    "scala": siScala,
    "haskell": siHaskell,
    "elixir": siElixir,
    "clojure": siClojure,
    "lua": siLua,
    "r": siR,
    "perl": siPerl,
    "svelte": siSvelte,
    "react": siReact,
    "jsx": siReact,
    "tsx": siReact,
    "vue": siVuedotjs,
    "vue.js": siVuedotjs,
    "html": siHtml5,
    "html5": siHtml5,
    "css": siCss,
    "scss": siCss,
    "less": siCss,
    "shell": siShell,
    "bash": siShell,
    "zsh": siShell,
    "powershell": siShell,
    "dockerfile": siDocker,
    "yaml": siYaml,
    "yml": siYaml,
    "json": siJson,
    "markdown": siMarkdown,
    "md": siMarkdown,
    "toml": siToml,
    "nix": siNixos,
    "zig": siZig
  };

  function getLanguageIcon(name: string): SimpleIcon | null {
    return LANGUAGE_ICON_MAP[name.trim().toLowerCase()] ?? null;
  }

  let repos = $state<RepoMetadata[]>([]);
  let searchQuery = $state("");
  let selectedLanguages = $state<string[]>([]);
  let viewMode = $state<'grid' | 'list'>('grid');
  let loading = $state(true);
  let isScanning = $state(false);
  let actionLoading = $state<Record<string, string | null>>({});
  let hoveredAction = $state<{ path: string; label: string; x: number; y: number } | null>(null);

  function hoverAction(e: MouseEvent, path: string, label: string) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    hoveredAction = { path, label, x: rect.left + rect.width / 2, y: rect.top };
  }
  let showLanguageDropdown = $state(false);
  let languageDropdownPosition = $state<{ top: number; left: number } | null>(null);
  let languageDropdownAnchor = $state<HTMLButtonElement | null>(null);
  let selectedRepoForPreview = $state<RepoMetadata | null>(null);
  let readmeContent = $state<{ html: string, raw: string }>({ html: "", raw: "" });
  let readmeLoading = $state(false);
  let previewMode = $state<'markdown' | 'unified'>('markdown');
  let previewRequestId = 0;
  let unlistenState: UnlistenFn;
  let unlistenStart: UnlistenFn;
  let unlistenEnd: UnlistenFn;
  let tagPopoverRepo: RepoMetadata | null = $state(null);
  let tagPopoverOpen = $state(false);
  let tagPopoverPosition = $state<{ top: number; left: number } | null>(null);
  let groupByMode = $state<'none' | 'language'>('none');
  let collapsedGroups = $state<Set<string>>(new Set());
  let groupVisibility = $state<Record<string, boolean>>({});
  let groupObserver: IntersectionObserver | null = null;
  let viewportWidth = $state(0);
  let appConfig = $state<{ watched_folders: string[]; group_by_mode?: string | null } | null>(null);
  let repoContextMenuRepo = $state<RepoMetadata | null>(null);
  let repoContextMenuPosition = $state<{ x: number; y: number } | null>(null);
  let installedEditors = $state<any[]>([]);

  function withTimeout<T>(promise: Promise<T>, ms: number, label: string): Promise<T> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`${label} timed out`)), ms);
      promise
        .then((value) => {
          clearTimeout(timer);
          resolve(value);
        })
        .catch((err) => {
          clearTimeout(timer);
          reject(err);
        });
    });
  }

  async function openPreview(repo: RepoMetadata) {
    selectedRepoForPreview = repo;
    readmeLoading = true;
    previewMode = 'markdown';
    const requestId = ++previewRequestId;
    const load = async () => {
      try {
        // Backend now returns pre-parsed HTML and RAW content
        const response = await invoke<{ html: string, raw: string }>("get_repo_readme", { path: repo.path });
        if (requestId !== previewRequestId) return;
        readmeContent = response;
      } catch (e) {
        if (requestId !== previewRequestId) return;
        readmeContent = { 
          html: "<h3>No README found</h3><p>This repository does not have a standard README file.</p>",
          raw: "No README found" 
        };
      } finally {
        if (requestId !== previewRequestId) return;
        readmeLoading = false;
        scheduleHighlight();
      }
    };

    if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
      (window as Window & { requestIdleCallback: (cb: () => void) => number }).requestIdleCallback(() => { void load(); });
    } else {
      setTimeout(() => { void load(); }, 0);
    }
  }

  function highlightCode() {
    const blocks = document.querySelectorAll('pre code');
    blocks.forEach((block) => {
      hljs.highlightElement(block as HTMLElement);
    });
  }

  function scheduleHighlight() {
    if (!readmeContent.html || previewMode !== 'markdown') return;
    const run = () => tick().then(() => highlightCode());
    if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
      (window as Window & { requestIdleCallback: (cb: () => void) => number }).requestIdleCallback(run);
    } else {
      setTimeout(run, 0);
    }
  }

  // Also highlights when mode changes to markdown
  $effect(() => {
    if (previewMode === 'markdown' && readmeContent.html && !readmeLoading) {
      scheduleHighlight();
    }
  });

  let isClosingPanel = $state(false);

  function closePreview() {
    if (isClosingPanel) return;
    isClosingPanel = true;
    // Increment request ID first to cancel any pending loads
    previewRequestId += 1;
    // Defer state change to next tick to avoid blocking the click event
    setTimeout(() => {
      selectedRepoForPreview = null;
      // Reset closing flag after transition completes
      setTimeout(() => {
        isClosingPanel = false;
      }, 300);
    }, 0);
  }

  function enhanceReadme(node: HTMLElement) {
    const applyImageHints = () => {
      const images = Array.from(node.querySelectorAll('img'));
      images.forEach((img) => {
        img.loading = 'lazy';
        img.decoding = 'async';
        img.fetchPriority = 'low';
        img.classList.add('readme-image');
      });
    };

    const handleLinkClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest("a") as HTMLAnchorElement | null;
      if (!anchor || !anchor.href) return;
      if (!anchor.href.startsWith("http")) return;
      event.preventDefault();
      event.stopPropagation();
      void openUrl(anchor.href);
    };

    applyImageHints();
    const observer = new MutationObserver(() => applyImageHints());
    observer.observe(node, { childList: true, subtree: true });
    node.addEventListener("click", handleLinkClick);

    return {
      destroy() {
        node.removeEventListener("click", handleLinkClick);
        observer.disconnect();
      }
    };
  }

  // Fuse.js for fuzzy search
  let fuse = $derived(new Fuse(repos, {
    keys: ['name', 'path', 'description'],
    threshold: 0.3
  }));

  const langStats = $derived.by(() => {
    const counts: Record<string, number> = {};
    repos.forEach(repo => {
      Object.keys(repo.languages || {}).forEach(lang => {
        counts[lang] = (counts[lang] || 0) + 1;
      });
    });
    return Object.entries(counts)
      .sort((a, b) => b[1] - a[1])
      .map(([lang, count]) => ({ name: lang, count }));
  });

  type RepoLanguageInfo = {
    primaryLanguage: string;
    sortedLanguages: Array<[string, number]>;
    languageCount: number;
  };

  const DEFAULT_LANGUAGE_INFO: RepoLanguageInfo = {
    primaryLanguage: "Other",
    sortedLanguages: [],
    languageCount: 0
  };

  const repoLanguageInfo = $derived.by(() => {
    const map = new Map<string, RepoLanguageInfo>();
    for (const repo of repos) {
      const entries = Object.entries(repo.languages || {});
      entries.sort((a, b) => {
        const diff = b[1] - a[1];
        if (diff !== 0) return diff;
        return a[0].localeCompare(b[0]);
      });
      const primaryLanguage = entries.length > 0 ? entries[0][0] : "Other";
      map.set(repo.path, {
        primaryLanguage,
        sortedLanguages: entries,
        languageCount: entries.length
      });
    }
    return map;
  });

  function getRepoLanguageInfo(repo: RepoMetadata | null | undefined): RepoLanguageInfo {
    if (!repo) return DEFAULT_LANGUAGE_INFO;
    return repoLanguageInfo.get(repo.path) ?? DEFAULT_LANGUAGE_INFO;
  }

  let languageFilterEl: HTMLDivElement | null = null;
  let languageMeasureEl: HTMLDivElement | null = null;
  let languageVisibleCount = $state(7);
  let languageResizeObserver: ResizeObserver | null = null;

  function recalcLanguageVisibleCount() {
    if (!languageFilterEl || !languageMeasureEl) return;
    const containerWidth = languageFilterEl.clientWidth;
    if (containerWidth <= 0) return;

    const style = getComputedStyle(languageFilterEl);
    const gap = parseFloat(style.columnGap || style.gap || "0");
    const allChip = languageMeasureEl.querySelector("[data-all-chip]") as HTMLElement | null;
    const langChips = Array.from(languageMeasureEl.querySelectorAll("[data-lang-chip]")) as HTMLElement[];
    const moreChip = languageMeasureEl.querySelector("[data-more-chip]") as HTMLElement | null;

    if (!allChip) return;

    const allWidth = allChip.getBoundingClientRect().width;
    const moreWidth = moreChip ? moreChip.getBoundingClientRect().width : 0;
    let used = allWidth;
    let count = 0;

    for (let i = 0; i < langChips.length; i += 1) {
      const chipWidth = langChips[i].getBoundingClientRect().width;
      const remaining = langChips.length - (count + 1);
      const needsMore = remaining > 0;
      const extra = needsMore ? gap + moreWidth : 0;
      const nextUsed = used + gap + chipWidth + extra;
      if (nextUsed <= containerWidth) {
        count += 1;
        used += gap + chipWidth;
      } else {
        break;
      }
    }

    languageVisibleCount = Math.max(0, Math.min(count, langStats.length));
  }

  function scheduleLanguageRecalc() {
    if (typeof window === "undefined") return;
    tick().then(() => requestAnimationFrame(recalcLanguageVisibleCount));
  }

  function toggleLanguageDropdown() {
    if (showLanguageDropdown) {
      showLanguageDropdown = false;
      languageDropdownPosition = null;
      return;
    }
    if (!languageDropdownAnchor) {
      showLanguageDropdown = true;
      return;
    }
    const rect = languageDropdownAnchor.getBoundingClientRect();
    const width = 224;
    const left = Math.min(rect.left, window.innerWidth - width - 16);
    languageDropdownPosition = {
      top: rect.bottom + 8,
      left: Math.max(16, left)
    };
    showLanguageDropdown = true;
  }

  onMount(() => {
    scheduleLanguageRecalc();
    if (typeof ResizeObserver !== "undefined" && languageFilterEl) {
      languageResizeObserver = new ResizeObserver(() => scheduleLanguageRecalc());
      languageResizeObserver.observe(languageFilterEl);
    }
  });

  onDestroy(() => {
    languageResizeObserver?.disconnect();
    languageResizeObserver = null;
  });

  $effect(() => {
    langStats;
    scheduleLanguageRecalc();
  });

  onMount(() => {
    if (typeof window === "undefined") return;
    viewportWidth = window.innerWidth;
    const handleResize = () => {
      viewportWidth = window.innerWidth;
    };
    window.addEventListener("resize", handleResize, { passive: true });
    return () => window.removeEventListener("resize", handleResize);
  });

  const visibleLanguages = $derived(langStats.slice(0, languageVisibleCount));
  const hiddenLanguages = $derived(langStats.slice(languageVisibleCount));

  const filteredRepos = $derived.by(() => {
    let result = searchQuery 
      ? fuse.search(searchQuery).map(r => r.item)
      : [...repos];
    
    if (selectedLanguages.length > 0) {
      result = result.filter(r => 
        selectedLanguages.some(lang => Object.keys(r.languages || {}).includes(lang))
      );
    }
    
    const selectedTagNames = $activeTagFilters;
    if (selectedTagNames.length > 0) {
      result = result.filter(r => 
        r.tags && r.tags.some(tag => selectedTagNames.includes(tag))
      );
    }
    
    return result;
  });

  const groupedRepos = $derived.by(() => {
    if (groupByMode !== 'language') return {};
    const groups: Record<string, RepoMetadata[]> = {};

    for (const repo of filteredRepos) {
      const key = getRepoLanguageInfo(repo).primaryLanguage;
      if (!groups[key]) groups[key] = [];
      groups[key].push(repo);
    }

    const sortedKeys = Object.keys(groups).sort((a, b) => a.localeCompare(b));
    const result: Record<string, RepoMetadata[]> = {};
    for (const k of sortedKeys) {
      if (groups[k].length > 0) {
        result[k] = groups[k];
      }
    }
    return result;
  });

  const gridColumns = $derived(() => {
    if (viewportWidth >= 1024) return 3;
    if (viewportWidth >= 768) return 2;
    return 1;
  });

  const LIST_ITEM_HEIGHT = 132;
  const LIST_GAP = 12;
  const GRID_ROW_HEIGHT = 360;
  const GRID_GAP = 24;

  function estimateGroupHeight(repoCount: number) {
    const rows = viewMode === 'grid'
      ? Math.max(1, Math.ceil(repoCount / gridColumns))
      : Math.max(1, repoCount);
    const rowHeight = viewMode === 'grid' ? GRID_ROW_HEIGHT : LIST_ITEM_HEIGHT;
    const gap = viewMode === 'grid' ? GRID_GAP : LIST_GAP;
    return rows * rowHeight + Math.max(0, rows - 1) * gap;
  }

  function observeGroup(node: HTMLElement, label: string) {
    if (typeof window === "undefined") return;
    if (!groupObserver) {
      groupObserver = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            const target = entry.target as HTMLElement;
            const key = target.dataset.groupLabel;
            if (!key) continue;
            if (entry.isIntersecting) {
              groupVisibility = { ...groupVisibility, [key]: true };
            }
          }
        },
        { rootMargin: "800px 0px 800px 0px", threshold: 0.01 }
      );
    }
    node.dataset.groupLabel = label;
    groupObserver.observe(node);
    return {
      destroy() {
        groupObserver?.unobserve(node);
      }
    };
  }

  // Keep repo tag badges in sync when a tag is deleted or renamed globally.
  $effect(() => {
    if ($tagLoading) return;

    const validTagNames = new Set($allTags.map((t) => t.name));
    let changed = false;

    const nextRepos = repos.map((repo) => {
      const filteredTags = (repo.tags || []).filter((tag) => validTagNames.has(tag));
      if (filteredTags.length !== (repo.tags || []).length) {
        changed = true;
        return { ...repo, tags: filteredTags };
      }
      return repo;
    });

    if (changed) {
      repos = nextRepos;
    }
  });

  function toggleLanguage(lang: string) {
    if (selectedLanguages.includes(lang)) {
      selectedLanguages = selectedLanguages.filter(l => l !== lang);
    } else {
      selectedLanguages = [...selectedLanguages, lang];
    }
  }

  function formatRelativeTime(timestamp: number) {
    if (timestamp === 0) return "Never";
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;
    if (diff < 60) return "Just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function getSyncStatusDetails(status: SyncStatus) {
    switch (status) {
      case 'Clean': return { icon: CheckCircle2, color: 'text-emerald-500', label: 'Clean' };
      case 'Ahead': return { icon: ArrowUp, color: 'text-primary', label: 'Ahead' };
      case 'Behind': return { icon: ArrowDown, color: 'text-amber-500', label: 'Behind' };
      case 'Dirty': return { icon: GitCommit, color: 'text-amber-500', label: 'Uncommitted' };
      case 'Diverged': return { icon: ArrowRightLeft, color: 'text-destructive', label: 'Diverged' };
      default: return { icon: AlertTriangle, color: 'text-muted-foreground', label: 'Unknown' };
    }
  }

  async function loadRepos() {
    try {
      repos = await withTimeout(invoke("list_repos"), 15000, "list_repos");
    } catch (e) {
      console.error("Failed to load repos:", e);
      toast.error("Failed to load repositories");
    }
  }

  async function refreshRepos() {
    if (isScanning) return;
    try {
      await invoke("refresh_repos");
    } catch (e) {
      toast.error("Scan Failed");
    }
  }

  async function loadInstalledEditors() {
    try {
      installedEditors = await invoke("get_installed_editors") as any[];
    } catch (e) {
      console.error("Failed to load editors:", e);
    }
  }

  function getEditorIcon(iconName: string): string | null {
    const key = `si${iconName.charAt(0).toUpperCase()}${iconName.slice(1).toLowerCase()}` as keyof typeof si;
    const icon = si[key] as SimpleIcon | undefined;
    return icon ? icon.svg : null;
  }

  function handleRepoContextMenu(event: MouseEvent, repo: RepoMetadata) {
    event.preventDefault();
    repoContextMenuRepo = repo;

    // Approximate menu height: header + finder + divider + editor-header + editors(max 180) + divider + git-fetch
    const menuHeight = 360;
    const menuWidth = 224; // w-56
    const padding = 8;

    let x = event.clientX;
    let y = event.clientY;

    if (y + menuHeight > window.innerHeight - padding) {
      y = Math.max(padding, window.innerHeight - menuHeight - padding);
    }
    if (x + menuWidth > window.innerWidth - padding) {
      x = Math.max(padding, window.innerWidth - menuWidth - padding);
    }

    repoContextMenuPosition = { x, y };
  }

  function closeRepoContextMenu() {
    repoContextMenuRepo = null;
    repoContextMenuPosition = null;
  }

  async function openInEditor(editorId: string, repoPath: string) {
    try {
      await invoke("open_in_editor", { editorId, path: repoPath });
      toast.success("Opening editor...");
      closeRepoContextMenu();
    } catch (e) {
      toast.error(`Failed: ${e}`);
    }
  }

  async function runGitAction(repo: RepoMetadata, action: 'fetch' | 'pull' | 'push') {
    const key = `${repo.path}-${action}`;
    actionLoading[key] = action;
    const promise = invoke<string>(`git_${action}`, { path: repo.path });
    toast.promise(promise, {
      loading: `Git ${action}ing ${repo.name}...`,
      success: () => {
        loadRepos();
        return `Git ${action} successful`;
      },
      error: (e: any) => `Git ${action} failed: ${typeof e === 'string' ? e : (e.GitError || e.IoError || "Unknown error")}`,
    });
    try { await promise; } finally { delete actionLoading[key]; }
  }

  async function openFolder(path: string) {
    try {
      await revealItemInDir(path);
    } catch (e) {
      toast.error("Failed to open folder");
    }
  }

  function clickOutside(node: HTMLElement, handler: () => void) {
    const onClick = (event: MouseEvent) => {
      if (node && !node.contains(event.target as Node) && !event.defaultPrevented) {
        handler();
      }
    };
    document.addEventListener('click', onClick, true);
    return {
      destroy() {
        document.removeEventListener('click', onClick, true);
      }
    };
  }

  onMount(async () => {
    loading = true;
    const configPromise = withTimeout(invoke<any>("get_config"), 8000, "get_config")
      .then((config) => {
        appConfig = config;
        if (config?.group_by_mode === "language") {
          groupByMode = 'language';
        }
      })
      .catch((e) => {
        console.error("Failed to load config:", e);
      });

    const loadTagsPromise = withTimeout(loadTags(), 8000, "load_tags").catch((e) => {
      console.error("Failed to load tags:", e);
    });

    void loadInstalledEditors();
    await Promise.allSettled([loadRepos(), loadTagsPromise, configPromise]);
    loading = false;

    try {
      isScanning = await withTimeout(invoke("is_scanning"), 4000, "is_scanning");
    } catch (e) {
      console.error("Failed to read scanning state:", e);
      isScanning = false;
    }
    unlistenState = await listen("repo-state-changed", () => loadRepos());
    unlistenStart = await listen("scan-started", () => {
      isScanning = true;
    });
    unlistenEnd = await listen("scan-completed", () => {
      isScanning = false;
      loadRepos();
    });
  });

  onDestroy(() => { 
    if (unlistenState) unlistenState();
    if (unlistenStart) unlistenStart();
    if (unlistenEnd) unlistenEnd();
    groupObserver?.disconnect();
    groupObserver = null;
  });

  async function setGroupByMode(mode: 'none' | 'language') {
    groupByMode = mode;
    try {
      const next = {
        watched_folders: appConfig?.watched_folders ?? [],
        group_by_mode: mode === 'none' ? null : 'language'
      };
      appConfig = next;
      await invoke("set_config", { config: next });
    } catch (e) {
      console.error("Failed to save group_by_mode:", e);
    }
  }

  function toggleGroupCollapse(label: string) {
    const next = new Set(collapsedGroups);
    if (next.has(label)) {
      next.delete(label);
      groupVisibility = { ...groupVisibility, [label]: true };
    } else {
      next.add(label);
    }
    collapsedGroups = next;
  }

  $effect(() => {
    if (groupByMode !== 'language') {
      if (Object.keys(groupVisibility).length > 0) {
        groupVisibility = {};
      }
      return;
    }
    const labels = Object.keys(groupedRepos);
    const prev = groupVisibility;
    const next: Record<string, boolean> = {};
    let changed = labels.length !== Object.keys(prev).length;
    for (const label of labels) {
      next[label] = prev[label] ?? false;
      if (next[label] !== prev[label]) changed = true;
    }
    if (changed) {
      groupVisibility = next;
    }
  });

  function openTagPopover(repo: RepoMetadata, anchorEl: HTMLElement) {
    tagPopoverRepo = repo;
    const rect = anchorEl.getBoundingClientRect();
    const popoverWidth = 288; // w-72
    const gap = 10;
    const viewportPadding = 12;

    const left = Math.min(
      Math.max(rect.right - popoverWidth, viewportPadding),
      window.innerWidth - popoverWidth - viewportPadding
    );
    const top = Math.max(rect.top - gap, viewportPadding);

    tagPopoverPosition = { top, left };
    tagPopoverOpen = true;
  }

  function closeTagPopover() {
    tagPopoverOpen = false;
    tagPopoverRepo = null;
    tagPopoverPosition = null;
  }

  function repoHasTag(repo: RepoMetadata, tag: StoreTag) {
    return repo.tags?.includes(tag.name);
  }

  async function toggleTagForRepo(repo: RepoMetadata, tag: StoreTag) {
    try {
      if (repoHasTag(repo, tag)) {
        await removeTag(repo.path, tag.id);
      } else {
        await assignTag(repo.path, tag.id);
      }

      // Sync tags for this repo from backend without requiring a full rescan
      const updated = await invoke<StoreTag[]>("get_repo_tags", { repoPath: repo.path });
      const tagNames = updated.map((t) => t.name);

      repos = repos.map((r) =>
        r.path === repo.path
          ? { ...r, tags: tagNames }
          : r
      );

      if (tagPopoverRepo?.path === repo.path) {
        tagPopoverRepo = { ...repo, tags: tagNames };
      }
    } catch (e: any) {
      const message = typeof e === "string" ? e : (e?.message || "Failed to update tag");
      toast.error(message);
    }
  }
</script>

{#snippet RepoCard(
  { repo, status }: {
    repo: RepoMetadata;
    status: { icon: typeof CheckCircle2; color: string; label: string };
  }
)}
  <Card
    class="group glass border-slate-200/70 shadow-none flex flex-col rounded-[1.5rem] overflow-hidden transition-[background-color,border-color,box-shadow,transform] duration-200 hover:bg-white hover:border-slate-300/90 hover:shadow-[0_10px_22px_rgba(15,23,42,0.09)] hover:-translate-y-0.5 {viewMode === 'list' ? 'flex-row items-center py-3 px-6' : ''} cursor-pointer"
    style=""
    onclick={(e) => {
      if ((e.target as HTMLElement).closest('button')) return;
      openPreview(repo);
    }}
    oncontextmenu={(e) => handleRepoContextMenu(e, repo)}
  >
    {@const langInfo = getRepoLanguageInfo(repo)}
    {#if viewMode === 'grid'}
      <CardContent class="p-8 space-y-8 flex-1 flex flex-col">
        <div class="flex items-start justify-between gap-6">
          <div class="space-y-2 flex-1 min-w-0">
            <h3 class="text-2xl font-black tracking-tight truncate group-hover:text-primary transition-colors">{repo.name}</h3>
          </div>
          <div class="p-3 bg-white/80 rounded-2xl text-muted-foreground group-hover:text-primary group-hover:bg-primary/10 transition-all duration-500">
            <GitBranch class="w-5 h-5" />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="bg-white/80 rounded-2xl p-3 border border-slate-200/70 hover:border-slate-200/80 transition-all">
            <div class="flex items-center space-x-2 mb-1">
              <GitBranch class="w-3 h-3 text-muted-foreground" />
              <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground">Branch</span>
            </div>
            <p class="text-xs font-bold truncate">{repo.branch}</p>
          </div>
          <div class="bg-white/80 rounded-2xl p-3 border border-slate-200/70 hover:border-slate-200/80 transition-all">
            <div class="flex items-center space-x-2 mb-1">
              <status.icon class="w-3 h-3 {status.color}" />
              <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground">Status</span>
            </div>
            <p class="text-xs font-bold {status.color}">{status.label}</p>
          </div>
        </div>

        <div class="flex flex-col gap-3">
          <div class="flex flex-wrap gap-1.5 min-h-[24px]">
            {#each langInfo.sortedLanguages.slice(0, 3) as [lang, count]}
            {@const icon = getLanguageIcon(lang)}
            <Badge variant="outline" class="bg-white/80 border-slate-200/70 text-[8px] px-2 py-0.5 rounded-md font-black uppercase tracking-widest text-muted-foreground">
              {#if icon}
                <svg viewBox="0 0 24 24" class="w-2.5 h-2.5 mr-1 inline-block align-[-2px]" style={`color: #${icon.hex}`} aria-label={`${lang} icon`}>
                  <path fill="currentColor" d={icon.path}></path>
                </svg>
              {/if}
              {lang}
            </Badge>
          {/each}
            {#if langInfo.languageCount > 3}
              <Badge variant="outline" class="bg-white/80 border-slate-200/70 text-[8px] px-2 py-0.5 rounded-md font-black">
                +{langInfo.languageCount - 3}
              </Badge>
            {/if}
          </div>

          <!-- Tag badges -->
          <div class="flex flex-wrap gap-1.5 min-h-[20px] mb-2">
            {#each repo.tags.slice(0, 3) as tagName}
              {@const match = $allTags.find(t => t.name === tagName)}
              <Badge
                variant="outline"
                class="bg-white/80 border-slate-200/80 text-[8px] px-2 py-0.5 rounded-full font-black uppercase tracking-widest flex items-center gap-1"
                style={`border-color: ${match?.color ?? '#6366f1'}33`}
              >
                <span
                  class="w-2 h-2 rounded-full"
                  style={`background: ${match?.color ?? '#6366f1'}`}
                ></span>
                {tagName}
              </Badge>
            {/each}
            {#if repo.tags.length > 3}
              <Badge variant="outline" class="bg-white/80 border-slate-200/80 text-[8px] px-2 py-0.5 rounded-full font-black">
                +{repo.tags.length - 3}
              </Badge>
            {/if}
          </div>

          {#if repo.description}
            <p class="text-[13px] text-muted-foreground line-clamp-3 min-h-[3rem] leading-relaxed font-medium">
              {repo.description}
            </p>
          {/if}
        </div>

        <div class="flex flex-wrap gap-2 pt-2">
          <Badge variant="outline" class="bg-white/80 border-slate-200/70 text-[10px] px-3 py-1 rounded-full font-bold">
            <Clock class="w-3 h-3 mr-1.5 text-primary" />
            {formatRelativeTime(repo.last_modified)}
          </Badge>
          {#if repo.remote_url}
            <Badge variant="outline" class="bg-white/80 border-slate-200/70 text-[10px] px-3 py-1 rounded-full font-bold {repo.remote_reachable ? 'text-emerald-500' : 'text-amber-500 opacity-80'}">
              <Globe class="w-3 h-3 mr-1.5" />
              {repo.remote_reachable ? 'Connected' : 'Unreachable'}
            </Badge>
          {/if}
        </div>

        <div class="mt-auto pt-8 flex items-center justify-between border-t border-slate-200/70 relative">
          <div class="flex items-center bg-white/85 rounded-full p-1.5 border border-slate-200/70 shadow-inner">
            <Button
              variant="ghost" size="icon" class="rounded-full h-11 w-11 transition-colors duration-200 hover:bg-emerald-50 hover:text-emerald-600"
              onmouseenter={(e) => hoverAction(e, repo.path, 'Fetch')}
              onmouseleave={() => hoveredAction = null}
              disabled={!!actionLoading[`${repo.path}-fetch`]}
              onclick={() => runGitAction(repo, 'fetch')}
            >
              <RefreshCw class="w-5 h-5 {actionLoading[`${repo.path}-fetch`] ? 'animate-spin' : ''}" />
            </Button>
            <Button
              variant="ghost" size="icon" class="rounded-full h-11 w-11 transition-colors duration-200 hover:bg-sky-50 hover:text-sky-600"
              onmouseenter={(e) => hoverAction(e, repo.path, 'Pull')}
              onmouseleave={() => hoveredAction = null}
              disabled={!!actionLoading[`${repo.path}-pull`]}
              onclick={() => runGitAction(repo, 'pull')}
            >
              <ArrowDown class="w-5 h-5 {actionLoading[`${repo.path}-pull`] ? 'animate-bounce' : ''}" />
            </Button>
            <Button
              variant="ghost" size="icon" class="rounded-full h-11 w-11 transition-colors duration-200 hover:bg-violet-50 hover:text-violet-600"
              onmouseenter={(e) => hoverAction(e, repo.path, 'Push')}
              onmouseleave={() => hoveredAction = null}
              disabled={!!actionLoading[`${repo.path}-push`]}
              onclick={() => runGitAction(repo, 'push')}
            >
              <ArrowUp class="w-5 h-5 {actionLoading[`${repo.path}-push`] ? 'animate-pulse text-violet-600' : ''}" />
            </Button>
          </div>

          <div class="flex items-center space-x-1">
            <Button
              variant="ghost"
              size="icon"
              class="rounded-full transition-colors duration-200 hover:bg-amber-50 hover:text-amber-600"
              onmouseenter={(e) => hoverAction(e, repo.path, 'Explore')}
              onmouseleave={() => hoveredAction = null}
              onclick={() => openFolder(repo.path)}
            >
              <FolderOpen class="w-5 h-5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="rounded-full transition-colors duration-200 hover:bg-primary/10 hover:text-primary"
              onmouseenter={(e) => hoverAction(e, repo.path, 'Tag')}
              onmouseleave={() => hoveredAction = null}
              onclick={(e) => openTagPopover(repo, e.currentTarget as HTMLElement)}
            >
              <Tag class="w-4 h-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    {:else}
      <!-- List Item -->
      <div class="flex-1 flex items-center justify-between py-3 px-5 overflow-hidden gap-6">
        <div class="flex items-center space-x-4 min-w-0 flex-1">
          <div class="p-2.5 bg-white/80 rounded-2xl text-muted-foreground group-hover:text-primary transition-all duration-500 group-hover:bg-primary/10">
            <GitBranch class="w-5 h-5" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center space-x-3 mb-2">
              <h3 class="font-bold truncate text-[15px] tracking-tight">{repo.name}</h3>
              <div class="flex gap-1.5">
                {#each langInfo.sortedLanguages.slice(0, 2) as [lang]}
                  {@const icon = getLanguageIcon(lang)}
                  <span class="inline-flex items-center text-[11px] px-2 py-0.5 bg-white/80 border border-slate-200/70 text-muted-foreground font-bold uppercase tracking-wide rounded-md">
                    {#if icon}
                      <svg viewBox="0 0 24 24" class="w-3 h-3 mr-1" style={`color: #${icon.hex}`} aria-label={`${lang} icon`}>
                        <path fill="currentColor" d={icon.path}></path>
                      </svg>
                    {/if}
                    {lang}
                  </span>
                {/each}
              </div>
            </div>
            {#if repo.description}
              <p class="text-[13px] text-muted-foreground font-medium truncate leading-snug mb-2">{repo.description}</p>
            {/if}
            <!-- Inline tag badges -->
            {#if repo.tags.length}
              <div class="flex flex-wrap gap-1.5 mt-1">
                {#each repo.tags.slice(0, 3) as tagName}
                  {@const match = $allTags.find(t => t.name === tagName)}
                  <span
                    class="inline-flex items-center gap-1.5 text-[11px] px-2 py-0.5 rounded-full bg-white/80 border border-slate-200/80 font-medium"
                    style={`border-color: ${match?.color ?? '#6366f1'}33`}
                  >
                    <span
                      class="w-2 h-2 rounded-full shrink-0"
                      style={`background: ${match?.color ?? '#6366f1'}`}
                    ></span>
                    {tagName}
                  </span>
                {/each}
                {#if repo.tags.length > 3}
                  <span class="text-[11px] px-2 py-0.5 rounded-full bg-white/80 border border-slate-200/80 font-bold text-muted-foreground">
                    +{repo.tags.length - 3}
                  </span>
                {/if}
              </div>
            {/if}
          </div>
        </div>

        <div class="flex items-center gap-6 flex-shrink-0">
          <div class="w-28 flex flex-col items-center gap-1">
            <span class="text-[11px] uppercase tracking-[0.2em] font-black text-muted-foreground">Branch</span>
            <span class="text-sm font-bold tracking-tight truncate max-w-[104px]">{repo.branch}</span>
          </div>
          <div class="w-28 flex flex-col items-center gap-1">
            <span class="text-[11px] uppercase tracking-[0.2em] font-black text-muted-foreground">Status</span>
            <div class="flex items-center space-x-1.5">
              <status.icon class="w-3.5 h-3.5 {status.color}" />
              <span class="text-sm font-bold tracking-tight {status.color}">{status.label}</span>
            </div>
          </div>
          <div class="w-28 flex flex-col items-center gap-1">
            <span class="text-[11px] uppercase tracking-[0.2em] font-black text-muted-foreground">Activity</span>
            <span class="text-sm font-bold text-muted-foreground tracking-tight">{formatRelativeTime(repo.last_modified)}</span>
          </div>
        </div>

        <div class="flex items-center space-x-2 flex-shrink-0 relative">
          <Button
            variant="ghost"
            size="icon"
            class="rounded-xl h-11 w-11 transition-colors duration-200 hover:bg-primary/10 hover:text-primary"
            onmouseenter={(e) => hoverAction(e, repo.path, 'Tag')}
            onmouseleave={() => hoveredAction = null}
            onclick={(e) => openTagPopover(repo, e.currentTarget as HTMLElement)}
          >
            <Tag class="w-4 h-4" />
          </Button>
          <Button
             variant="ghost" size="icon" class="rounded-xl h-11 w-11 transition-colors duration-200 hover:bg-emerald-50 hover:text-emerald-600"
             onmouseenter={(e) => hoverAction(e, repo.path, 'Fetch')}
             onmouseleave={() => hoveredAction = null}
             disabled={!!actionLoading[`${repo.path}-fetch`]}
             onclick={() => runGitAction(repo, 'fetch')}
          >
            <RefreshCw class="w-5 h-5 {actionLoading[`${repo.path}-fetch`] ? 'animate-spin' : ''}" />
          </Button>
          <Button
             variant="ghost" size="icon" class="rounded-xl h-11 w-11 transition-colors duration-200 hover:bg-sky-50 hover:text-sky-600"
             onmouseenter={(e) => hoverAction(e, repo.path, 'Pull')}
             onmouseleave={() => hoveredAction = null}
             disabled={!!actionLoading[`${repo.path}-pull`]}
             onclick={() => runGitAction(repo, 'pull')}
          >
            <ArrowDown class="w-5 h-5 {actionLoading[`${repo.path}-pull`] ? 'animate-bounce' : ''}" />
          </Button>
          <Button
             variant="ghost" size="icon" class="rounded-xl h-11 w-11 transition-colors duration-200 hover:bg-amber-50 hover:text-amber-600"
             onmouseenter={(e) => hoverAction(e, repo.path, 'Explore')}
             onmouseleave={() => hoveredAction = null}
             onclick={() => openFolder(repo.path)}
          >
            <FolderOpen class="w-5 h-5" />
          </Button>
        </div>
      </div>
    {/if}
  </Card>
{/snippet}

<div class="p-8 space-y-8 max-w-7xl mx-auto">
  <!-- Header / Filters -->
  <div class="space-y-5">
    <div>
      <h1 class="text-5xl font-black tracking-tighter text-glow mb-2">Vessels in Dock</h1>
      <p class="text-muted-foreground text-xl">Managing <span class="text-primary font-bold">{repos.length}</span> repositories.</p>
    </div>

    <div class="flex flex-nowrap items-center gap-4 overflow-x-hidden overflow-y-visible">
      <div class="flex flex-nowrap items-center gap-3 flex-1 min-w-0">
        <!-- Fuzzy Search -->
        <div class="relative group w-[260px] md:w-[320px] shrink-0">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
          <input 
            type="text" 
            bind:value={searchQuery}
            placeholder="Fuzzy search projects..." 
            class="bg-white/80 border border-slate-200/80 rounded-xl pl-10 pr-4 py-2 text-sm w-full focus:outline-none focus:ring-2 focus:ring-primary/40 focus:bg-white/10 transition-all font-medium"
          />
        </div>

        <!-- Language Filter -->
        <div
          class="relative flex flex-nowrap items-center gap-1.5 bg-white/80 rounded-xl border border-slate-200/80 p-1.5 max-w-full min-w-0 flex-1"
          bind:this={languageFilterEl}
        >
          <div class="flex flex-nowrap items-center gap-1.5 overflow-hidden min-w-0 flex-1">
            <button 
              onclick={() => selectedLanguages = []}
              class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all border border-transparent shrink-0 {selectedLanguages.length === 0 ? 'bg-primary text-primary-foreground shadow-glow border-primary/20' : 'hover:bg-slate-100 text-muted-foreground'}"
            >
              All
            </button>
            
            {#each visibleLanguages as lang}
              {@const icon = getLanguageIcon(lang.name)}
              <button 
                onclick={() => toggleLanguage(lang.name)}
                class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all border whitespace-nowrap shrink-0 {selectedLanguages.includes(lang.name) ? 'bg-white/10 text-primary border-primary/40 shadow-glow' : 'hover:bg-slate-100 text-muted-foreground border-transparent'}"
              >
                <div class="flex items-center gap-2">
                  {#if icon}
                    <svg viewBox="0 0 24 24" class="w-3 h-3 shrink-0" style={`color: #${icon.hex}`} aria-label={`${lang.name} icon`}>
                      <path fill="currentColor" d={icon.path}></path>
                    </svg>
                  {/if}
                  {lang.name}
                  <span class="opacity-40 text-[8px] font-medium">{lang.count}</span>
                  {#if selectedLanguages.includes(lang.name)}
                    <Check class="w-3 h-3" />
                  {/if}
                </div>
              </button>
            {/each}
          </div>

          {#if hiddenLanguages.length > 0}
            <div class="shrink-0">
              <button 
                bind:this={languageDropdownAnchor}
                onclick={toggleLanguageDropdown}
                class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all border whitespace-nowrap {hiddenLanguages.some(l => selectedLanguages.includes(l.name)) ? 'bg-white/10 text-primary border-primary/40' : 'hover:bg-slate-100 text-muted-foreground border-transparent'}"
              >
                +{hiddenLanguages.length}
              </button>
            </div>
          {/if}
        </div>

        {#if showLanguageDropdown && languageDropdownPosition}
          <div 
            use:clickOutside={() => { showLanguageDropdown = false; languageDropdownPosition = null; }}
            class="fixed w-56 bg-background/98 border border-slate-200/80 rounded-2xl shadow-[0_20px_50px_rgba(0,0,0,0.5)] backdrop-blur-2xl z-[110] p-3 animate-in fade-in zoom-in-95 slide-in-from-top-2 duration-200 ring-1 ring-slate-200/70"
            style={`top: ${languageDropdownPosition.top}px; left: ${languageDropdownPosition.left}px;`}
          >
            <div class="px-3 py-2 mb-2 border-b border-slate-200/70">
              <span class="text-[9px] font-black uppercase tracking-[0.2em] text-muted-foreground">More Languages</span>
            </div>
            <div class="max-h-64 overflow-y-auto space-y-1 custom-scrollbar pr-1">
              {#each hiddenLanguages as lang}
                {@const icon = getLanguageIcon(lang.name)}
                <button 
                  onclick={(e) => { e.stopPropagation(); toggleLanguage(lang.name); showLanguageDropdown = false; languageDropdownPosition = null; }}
                  class="w-full text-left px-3 py-2.5 text-[10px] font-black uppercase tracking-widest rounded-xl hover:bg-slate-100 transition-all flex items-center justify-between group {selectedLanguages.includes(lang.name) ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground'}"
                >
                  <div class="flex items-center gap-2 truncate">
                    {#if icon}
                      <svg viewBox="0 0 24 24" class="w-3 h-3 shrink-0" style={`color: #${icon.hex}`} aria-label={`${lang.name} icon`}>
                        <path fill="currentColor" d={icon.path}></path>
                      </svg>
                    {/if}
                    <span>{lang.name}</span>
                    <span class="opacity-40 text-[8px] font-medium">{lang.count}</span>
                  </div>
                  {#if selectedLanguages.includes(lang.name)}
                    <Check class="w-3.5 h-3.5 text-primary" />
                  {:else}
                    <div class="w-1.5 h-1.5 rounded-full bg-white/80 group-hover:bg-primary/40 transition-colors"></div>
                  {/if}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <div
          class="absolute -left-[9999px] -top-[9999px] opacity-0 pointer-events-none"
          aria-hidden="true"
          bind:this={languageMeasureEl}
        >
          <div class="flex flex-nowrap items-center gap-1.5">
            <button data-all-chip class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg border border-transparent">
              All
            </button>
            {#each langStats as lang}
              {@const icon = getLanguageIcon(lang.name)}
              <button data-lang-chip class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg border whitespace-nowrap">
                <div class="flex items-center gap-2">
                  {#if icon}
                    <svg viewBox="0 0 24 24" class="w-3 h-3 shrink-0" aria-hidden="true">
                      <path fill="currentColor" d={icon.path}></path>
                    </svg>
                  {/if}
                  {lang.name}
                  <span class="opacity-40 text-[8px] font-medium">{lang.count}</span>
                </div>
              </button>
            {/each}
            <button data-more-chip class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg border whitespace-nowrap">
              +999
            </button>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3 shrink-0 ml-auto">
        <div class="bg-white/80 rounded-xl border border-slate-200/80 p-1 flex items-center">
          <button 
            onclick={() => viewMode = 'grid'}
            class="p-2 rounded-lg transition-all {viewMode === 'grid' ? 'bg-white/10 text-primary shadow-inner' : 'text-muted-foreground hover:bg-slate-100'}"
          >
            <LayoutGrid class="w-4 h-4" />
          </button>
          <button 
            onclick={() => viewMode = 'list'}
            class="p-2 rounded-lg transition-all {viewMode === 'list' ? 'bg-white/10 text-primary shadow-inner' : 'text-muted-foreground hover:bg-slate-100'}"
          >
            <LayoutList class="w-4 h-4" />
          </button>
        </div>
        
        <div class="bg-white/80 rounded-xl border border-slate-200/80 p-1 flex items-center gap-1">
          <span class="px-2 text-[9px] uppercase tracking-[0.25em] text-muted-foreground">Group By</span>
          <button 
            onclick={() => setGroupByMode('none')}
            class="px-3 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all {groupByMode === 'none' ? 'bg-white/10 text-primary shadow-inner' : 'text-muted-foreground hover:bg-slate-100'}"
          >
            None
          </button>
          <button 
            onclick={() => setGroupByMode('language')}
            class="px-3 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg transition-all {groupByMode === 'language' ? 'bg-white/10 text-primary shadow-inner' : 'text-muted-foreground hover:bg-slate-100'}"
          >
            Language
          </button>
        </div>

        <Button variant="outline" size="icon" class="rounded-xl border-slate-200/80" onclick={refreshRepos} disabled={isScanning}>
          <RefreshCw class="w-4 h-4 {isScanning ? 'animate-spin text-primary' : ''}" />
        </Button>
      </div>
    </div>
  </div>

  {#if loading && repos.length === 0}
    <!-- Loading state... -->
    <div class="py-32 flex flex-col items-center justify-center space-y-6">
      <div class="relative w-16 h-16">
        <div class="absolute inset-0 border-4 border-primary/20 rounded-full"></div>
        <div class="absolute inset-0 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
      </div>
      <p class="text-sm font-black uppercase tracking-[0.2em] text-primary/60 animate-pulse">Scanning Horizon</p>
    </div>
  {:else if filteredRepos.length === 0}
    <div class="py-32 glass rounded-3xl flex flex-col items-center justify-center space-y-8 text-center px-6">
      <div class="relative p-8 rounded-full bg-white/80 border border-slate-200/80">
        <Search class="w-16 h-16 text-primary opacity-20" />
      </div>
      <div class="space-y-3">
        <h3 class="text-3xl font-bold">No Vessels Found</h3>
        <p class="text-muted-foreground max-w-sm mx-auto text-lg">
          No repositories match your search or filter criteria.
        </p>
      </div>
      <Button variant="outline" class="rounded-full px-8" onclick={() => { searchQuery = ""; selectedLanguages = []; }}>Reset Filters</Button>
    </div>
  {:else}
    {#if groupByMode === 'language'}
      <div class="space-y-6">
        {#each Object.entries(groupedRepos) as [label, group]}
          <div use:observeGroup={label}>
            <GroupHeader
              label={label}
              count={group.length}
              collapsed={collapsedGroups.has(label)}
              onToggle={() => toggleGroupCollapse(label)}
            >
              {#snippet children()}
                {#if groupVisibility[label]}
                  <div class={viewMode === 'grid' ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-3" : "space-y-3 mt-3"}>
                    {#each group as repo (repo.path)}
                      {@const status = getSyncStatusDetails(repo.sync_status)}
                      {@render RepoCard({ repo, status })}
                    {/each}
                  </div>
                {:else}
                  <div style={`height: ${estimateGroupHeight(group.length)}px;`}></div>
                {/if}
              {/snippet}
            </GroupHeader>
          </div>
        {/each}
      </div>
    {:else}
      <div class={viewMode === 'grid' ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6" : "space-y-3"}>
        {#each filteredRepos as repo (repo.path)}
          {@const status = getSyncStatusDetails(repo.sync_status)}
          {@render RepoCard({ repo, status })}
        {/each}
      </div>
    {/if}
  {/if}
</div>

{#if tagPopoverOpen && tagPopoverRepo && tagPopoverPosition}
  <button 
    type="button"
    class="fixed inset-0 bg-slate-900/20 backdrop-blur-[2px] z-[80]"
    onclick={closeTagPopover}
    aria-label="Close tag selector"
  ></button>
  <div
    class="fixed w-72 bg-background/95 border border-slate-200/80 rounded-2xl shadow-2xl z-[81] p-4 space-y-3 animate-in fade-in zoom-in-95 duration-200"
    style={`top: ${tagPopoverPosition.top}px; left: ${tagPopoverPosition.left}px; transform: translateY(-100%);`}
    use:clickOutside={closeTagPopover}
  >
    <div class="flex items-center justify-between mb-1">
      <div class="space-y-0.5">
        <p class="text-[10px] uppercase tracking-[0.25em] text-muted-foreground font-black">Tags</p>
        <p class="text-xs font-semibold truncate">{tagPopoverRepo.name}</p>
      </div>
      <button
        class="w-6 h-6 rounded-full bg-white/80 hover:bg-slate-100 flex items-center justify-center text-muted-foreground"
        onclick={closeTagPopover}
      >
        <X class="w-3 h-3" />
      </button>
    </div>

    {#if !$allTags.length}
      <p class="text-[11px] text-muted-foreground">No tags yet. Create one from the sidebar first.</p>
    {:else}
      <div class="max-h-64 overflow-y-auto space-y-1 pr-1">
        {#each $allTags as tag}
          {@const checked = repoHasTag(tagPopoverRepo, tag)}
          <button
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-[11px] hover:bg-slate-100 border border-transparent {checked ? 'text-foreground' : ''}"
            style={checked ? `background-color: ${tag.color}22` : ''}
            onclick={() => tagPopoverRepo && toggleTagForRepo(tagPopoverRepo, tag)}
          >
            <div class="flex items-center gap-2 min-w-0">
              <span
                class="w-2 h-2 rounded-full border border-slate-300"
                style={`background: ${tag.color}`}
              ></span>
              <span class="truncate">{tag.name}</span>
            </div>
            <span class="text-[10px] text-muted-foreground">
              {#if checked}✔{:else}+{/if}
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<!-- Side Panel (README Preview) -->
{#if selectedRepoForPreview}
  <button
    type="button"
    class="fixed inset-0 bg-slate-900/20 z-[100] cursor-default"
    onclick={closePreview}
    aria-label="Close preview"
    in:fade={{ duration: 200 }}
  ></button>
  <div
    class="fixed top-0 right-0 h-full w-[45%] lg:w-[50%] bg-background border-l border-slate-200/80 z-[101] flex flex-col shadow-[0_0_40px_rgba(0,0,0,0.1)] will-change-transform"
    in:fly={{ x: '100%', duration: 350, opacity: 1, easing: cubicOut }}
    out:fly={{ x: '100%', duration: 280, opacity: 1, easing: cubicOut }}
  >
    <!-- Header -->
    <div class="p-6 border-b border-slate-200/70 flex items-center justify-between bg-white/80">
      <div class="space-y-1">
        <h2 class="text-xl font-black uppercase tracking-widest">{selectedRepoForPreview.name}</h2>
      </div>
      <Button variant="ghost" size="icon" class="rounded-xl hover:bg-slate-100" onclick={closePreview}>
        <X class="w-5 h-5" />
      </Button>
    </div>

    {#if readmeLoading}
      <div class="flex-1 flex flex-col items-center justify-center space-y-6 py-48" in:fade={{ duration: 200 }}>
        <div class="relative w-16 h-16">
          <RefreshCw class="w-full h-full animate-spin text-primary opacity-20" />
          <div class="absolute inset-0 flex items-center justify-center">
            <FileText class="w-6 h-6 text-primary" />
          </div>
        </div>
        <p class="text-xs font-black uppercase tracking-[0.4em] text-muted-foreground animate-pulse">Decrypting Repository Core</p>
      </div>
    {:else if readmeContent.html}
      <div class="flex-1 flex flex-col overflow-hidden" in:fade={{ duration: 200, delay: 50 }}>
        <!-- Mode Selector -->
        <div class="px-6 py-4 flex items-center justify-between border-b border-slate-200/70 bg-white/[0.02]">
          <div class="flex items-center space-x-2 bg-white/80 p-1 rounded-xl border border-slate-200/80">
            <button 
              onclick={() => previewMode = 'markdown'}
              class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-lg transition-all {previewMode === 'markdown' ? 'bg-primary text-primary-foreground shadow-glow' : 'text-muted-foreground hover:text-foreground'}"
            >
              Markdown
            </button>
            <button 
              onclick={() => previewMode = 'unified'}
              class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest rounded-lg transition-all {previewMode === 'unified' ? 'bg-primary text-primary-foreground shadow-glow' : 'text-muted-foreground hover:text-foreground'}"
            >
              Unified
            </button>
          </div>
          
          <div class="flex items-center gap-2">
             <OpenInEditor 
               repoPath={selectedRepoForPreview.path}
               variant="outline"
               size="sm"
               class="h-8 rounded-lg border-slate-200/80 text-[10px] font-bold uppercase tracking-widest bg-white/80 hover:bg-slate-100"
               externalEditors={installedEditors}
             />
             <Button variant="outline" size="sm" class="h-8 rounded-lg border-slate-200/80 text-[10px] font-bold uppercase tracking-widest bg-white/80 hover:bg-slate-100" onclick={() => selectedRepoForPreview && openFolder(selectedRepoForPreview.path)}>
               <FolderOpen class="w-3 h-3 mr-2 text-primary" />
               Open Folder
             </Button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto p-12 prose prose-lg prose-neutral max-w-none no-scrollbar 
          prose-a:text-primary prose-a:no-underline hover:prose-a:underline
          prose-blockquote:border-l-primary prose-blockquote:bg-primary/5 prose-blockquote:rounded-r-2xl prose-blockquote:py-2
          prose-img:rounded-3xl prose-img:border prose-img:border-slate-200/80 prose-img:shadow-2xl">
          
          {#if previewMode === 'markdown'}
            <div class="animate-in fade-in slide-in-from-bottom-2 duration-300" use:enhanceReadme>
              {@html readmeContent.html}
            </div>
          {:else}
            <div class="space-y-10 animate-in fade-in duration-200 pb-20">
              <div class="space-y-4">
                <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Metadata Summary</h3>
                <div class="grid grid-cols-2 gap-4">
                   <div class="bg-white/90 p-5 rounded-2xl border border-slate-200/70">
                     <p class="text-[9px] text-muted-foreground uppercase font-black mb-1.5 tracking-widest">Active Branch</p>
                     <div class="flex items-center gap-2.5">
                       <div class="p-1.5 bg-primary/10 rounded-lg text-primary">
                         <GitBranch class="w-4 h-4" />
                       </div>
                       <span class="text-sm font-bold tracking-tight">{selectedRepoForPreview.branch}</span>
                     </div>
                   </div>
                   <div class="bg-white/90 p-5 rounded-2xl border border-slate-200/70">
                     <p class="text-[9px] text-muted-foreground uppercase font-black mb-1.5 tracking-widest">Dock Progress</p>
                     <div class="flex items-center gap-2.5">
                       <div class="p-1.5 bg-amber-500/10 rounded-lg text-amber-500">
                         <Clock class="w-4 h-4" />
                       </div>
                       <span class="text-sm font-bold tracking-tight">{getSyncStatusDetails(selectedRepoForPreview.sync_status).label}</span>
                     </div>
                   </div>
                </div>
              </div>
              
              <div class="space-y-4">
                 <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Manifested Artifacts</h3>
                 <div class="flex flex-wrap gap-2.5">
                   {#each getRepoLanguageInfo(selectedRepoForPreview).sortedLanguages as [lang, count]}
                     {@const icon = getLanguageIcon(lang)}
                     <div class="bg-white/90 px-5 py-3 rounded-2xl border border-slate-200/70 flex items-center gap-4 hover:border-primary/20 transition-colors">
                       {#if icon}
                         <svg viewBox="0 0 24 24" class="w-4 h-4 shrink-0" style={`color: #${icon.hex}`} aria-label={`${lang} icon`}>
                           <path fill="currentColor" d={icon.path}></path>
                         </svg>
                       {:else}
                         <div class="w-2 h-2 rounded-full bg-primary/40"></div>
                       {/if}
                       <span class="text-xs font-bold uppercase tracking-wider">{lang}</span>
                       <span class="text-[9px] font-black p-1 bg-white/80 rounded border border-slate-200/70 text-muted-foreground">{count}</span>
                     </div>
                   {/each}
                 </div>
              </div>

              {#if selectedRepoForPreview.description}
                <div class="space-y-4">
                   <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Intelligence</h3>
                   <div class="bg-primary/5 p-6 rounded-3xl border border-primary/10 italic text-sm text-muted-foreground leading-relaxed">
                     "{selectedRepoForPreview.description}"
                   </div>
                </div>
              {/if}

              <div class="space-y-4">
                 <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Tags</h3>
                 <div class="flex flex-wrap gap-2">
                   {#if selectedRepoForPreview.tags?.length}
                     {#each selectedRepoForPreview.tags as tagName}
                       {@const match = $allTags.find(t => t.name === tagName)}
                       <span
                         class="inline-flex items-center gap-2 text-[10px] px-3 py-1 rounded-full bg-white/90 border border-slate-200/80 font-semibold text-muted-foreground"
                         style={`border-color: ${match?.color ?? '#6366f1'}55`}
                       >
                         <span
                           class="w-2 h-2 rounded-full"
                           style={`background: ${match?.color ?? '#6366f1'}`}
                         ></span>
                         {tagName}
                       </span>
                     {/each}
                   {:else}
                     <span class="text-xs text-muted-foreground">No tags assigned</span>
                   {/if}
                 </div>
              </div>

              <div class="space-y-4">
                 <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Local Directory</h3>
                 <div class="bg-white/90 p-5 rounded-2xl border border-slate-200/70 flex items-center justify-between">
                   <div class="flex items-center gap-3 min-w-0">
                     <FolderOpen class="w-4 h-4 text-muted-foreground" />
                     <span class="text-xs font-mono text-muted-foreground truncate max-w-xs">{selectedRepoForPreview.path}</span>
                   </div>
                 </div>
              </div>
              
              <div class="space-y-4">
                 <h3 class="text-xs font-black uppercase tracking-[0.2em] text-primary border-l-2 border-primary pl-4">Remote Link</h3>
                 <div class="bg-white/90 p-5 rounded-2xl border border-slate-200/70 flex items-center justify-between">
                   <div class="flex items-center gap-3">
                      <Globe class="w-4 h-4 text-muted-foreground" />
                      {#if selectedRepoForPreview.remote_url}
                        <a
                          href={selectedRepoForPreview.remote_url}
                          target="_blank"
                          rel="noreferrer"
                          class="text-xs font-mono text-primary truncate max-w-xs hover:underline underline-offset-4"
                        >
                          {selectedRepoForPreview.remote_url}
                        </a>
                      {:else}
                        <span class="text-xs font-mono text-muted-foreground truncate max-w-xs">No Remote Configured</span>
                      {/if}
                   </div>
                   {#if selectedRepoForPreview.remote_url}
                      <Badge variant="outline" class="{selectedRepoForPreview.remote_reachable ? 'bg-emerald-500/10 text-emerald-500' : 'bg-destructive/10 text-destructive'} border-transparent">
                        {selectedRepoForPreview.remote_reachable ? 'Online' : 'Offline'}
                      </Badge>
                   {/if}
                 </div>
              </div>
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <div class="flex-1 flex flex-col items-center justify-center space-y-6 py-48" in:fade={{ duration: 200 }}>
        <div class="relative w-16 h-16">
          <FileText class="w-full h-full text-primary opacity-20" />
          <div class="absolute inset-0 flex items-center justify-center">
            <RefreshCw class="w-6 h-6 text-primary animate-spin" />
          </div>
        </div>
        <p class="text-xs font-black uppercase tracking-[0.4em] text-muted-foreground animate-pulse">No README Found</p>
      </div>
    {/if}
  </div>
{/if}

{#if repoContextMenuRepo && repoContextMenuPosition}
  <button 
    type="button"
    aria-label="Close repo context menu"
    class="fixed inset-0 z-[110]"
    onclick={closeRepoContextMenu}
    oncontextmenu={(e) => { e.preventDefault(); closeRepoContextMenu(); }}
  ></button>
  <div
    class="fixed z-[111] w-56 bg-white/95 backdrop-blur-md border border-slate-200 rounded-2xl shadow-2xl py-2 text-xs overflow-hidden animate-in fade-in zoom-in-95 duration-150"
    style={`top: ${repoContextMenuPosition.y}px; left: ${repoContextMenuPosition.x}px;`}
  >
    <div class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400">
      Repository Actions
    </div>
    
    <button
      class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-primary/5 text-left transition-colors group"
      onclick={() => { repoContextMenuRepo && openFolder(repoContextMenuRepo.path); closeRepoContextMenu(); }}
    >
      <FolderOpen class="w-4 h-4 text-slate-400 group-hover:text-primary" />
      <span class="font-medium text-slate-700 group-hover:text-primary">Show in Finder</span>
    </button>

    <div class="h-px bg-slate-100 my-1"></div>

    <div class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400">
      Open In Editor
    </div>

    <div class="overflow-y-auto max-h-[180px] overscroll-contain [&::-webkit-scrollbar]:w-1 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-slate-200 [&::-webkit-scrollbar-thumb]:rounded-full">
      {#each installedEditors as editor}
        <button
          class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-primary/5 text-left transition-colors group"
          onclick={() => repoContextMenuRepo && openInEditor(editor.id, repoContextMenuRepo.path)}
        >
          <div class="w-4 h-4 flex items-center justify-center opacity-50 group-hover:opacity-100 shrink-0">
            {#if getEditorIcon(editor.icon)}
              <div class="w-3.5 h-3.5 fill-current">{@html getEditorIcon(editor.icon)}</div>
            {:else}
              <Code2 class="w-3.5 h-3.5 text-slate-400 group-hover:text-primary" />
            {/if}
          </div>
          <span class="font-medium text-slate-700 group-hover:text-primary">{editor.name}</span>
        </button>
      {:else}
        <div class="px-3 py-2 text-slate-400 italic">No editors detected</div>
      {/each}
    </div>

    <div class="h-px bg-slate-100 my-1"></div>

    <button
      class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-primary/5 text-left transition-colors group"
      onclick={() => { repoContextMenuRepo && runGitAction(repoContextMenuRepo, 'fetch'); closeRepoContextMenu(); }}
    >
      <RefreshCw class="w-4 h-4 text-slate-400 group-hover:text-primary" />
      <span class="font-medium text-slate-700 group-hover:text-primary">Git Fetch</span>
    </button>
  </div>
{/if}

<!-- Global action tooltip — fixed so it escapes overflow-hidden cards -->
{#if hoveredAction}
  <div
    class="fixed pointer-events-none z-[9999] -translate-x-1/2 bg-primary px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest text-primary-foreground shadow-glow animate-in fade-in zoom-in-95 duration-150 whitespace-nowrap"
    style="left: {hoveredAction.x}px; top: {hoveredAction.y - 36}px;"
  >
    {hoveredAction.label}
  </div>
{/if}
