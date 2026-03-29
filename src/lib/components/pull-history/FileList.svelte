<script lang="ts">
  import type { PullHistoryFile } from "$lib/stores/pullHistoryStore";
  import { loadFileDiff } from "$lib/stores/pullHistoryStore";
  import {
    ChevronRight,
    FileText,
    Folder,
    FolderOpen,
    Loader2,
  } from "lucide-svelte";
  import { tick, untrack } from "svelte";
  import FileDiffViewer from "./FileDiffViewer.svelte";

  interface Props {
    files: PullHistoryFile[];
    totalAdditions: number;
    totalDeletions: number;
    /** When set, expands the containing directory and scrolls to that file row. */
    highlightFileId?: number | null;
  }

  let {
    files,
    totalAdditions,
    totalDeletions,
    highlightFileId = null,
  }: Props = $props();

  // ── Tree types ────────────────────────────────────────────────────────────────
  interface FileNode {
    type: "file";
    name: string;
    path: string;
    file: PullHistoryFile;
    additions: number;
    deletions: number;
    fileCount: 1;
  }

  interface DirNode {
    type: "dir";
    name: string;
    path: string;
    children: TreeNode[];
    additions: number;
    deletions: number;
    fileCount: number;
  }

  type TreeNode = FileNode | DirNode;

  // ── Tree construction ─────────────────────────────────────────────────────────
  function insertFile(
    parent: DirNode,
    file: PullHistoryFile,
    parts: string[],
    depth: number,
  ) {
    if (depth === parts.length - 1) {
      parent.children.push({
        type: "file",
        name: parts[depth],
        path: file.file_path,
        file,
        additions: file.additions,
        deletions: file.deletions,
        fileCount: 1,
      });
      return;
    }
    const dirName = parts[depth];
    const dirPath = parts.slice(0, depth + 1).join("/");
    let dir = parent.children.find(
      (c): c is DirNode => c.type === "dir" && c.name === dirName,
    );
    if (!dir) {
      dir = {
        type: "dir",
        name: dirName,
        path: dirPath,
        children: [],
        additions: 0,
        deletions: 0,
        fileCount: 0,
      };
      parent.children.push(dir);
    }
    insertFile(dir, file, parts, depth + 1);
  }

  function computeStats(node: DirNode) {
    node.additions = 0;
    node.deletions = 0;
    node.fileCount = 0;
    for (const child of node.children) {
      if (child.type === "file") {
        node.additions += child.additions;
        node.deletions += child.deletions;
        node.fileCount++;
      } else {
        computeStats(child);
        node.additions += child.additions;
        node.deletions += child.deletions;
        node.fileCount += child.fileCount;
      }
    }
  }

  function sortChildren(node: DirNode) {
    node.children.sort((a, b) =>
      a.type !== b.type
        ? a.type === "dir"
          ? -1
          : 1
        : a.name.localeCompare(b.name),
    );
    node.children.forEach((c) => c.type === "dir" && sortChildren(c));
  }

  function collectDirPaths(nodes: TreeNode[]): string[] {
    return nodes.flatMap((n) =>
      n.type === "dir" ? [n.path, ...collectDirPaths(n.children)] : [],
    );
  }

  const tree = $derived.by(() => {
    const root: DirNode = {
      type: "dir",
      name: "",
      path: "",
      children: [],
      additions: 0,
      deletions: 0,
      fileCount: 0,
    };
    for (const file of files)
      insertFile(root, file, file.file_path.split("/"), 0);
    computeStats(root);
    sortChildren(root);
    return root.children;
  });

  // ── Collapse state — all dirs start collapsed ─────────────────────────────────
  let collapsedDirs = $state<Set<string>>(new Set());
  let initialized = $state(false);

  $effect(() => {
    if (!initialized && tree.length > 0) {
      collapsedDirs = new Set(collectDirPaths(tree));
      initialized = true;
    }
  });

  function toggleDir(path: string) {
    const next = new Set(collapsedDirs);
    next.has(path) ? next.delete(path) : next.add(path);
    collapsedDirs = next;
  }

  // ── Scroll-to-file from AI summary link ──────────────────────────────────────
  $effect(() => {
    if (!highlightFileId) return;
    const file = files.find((f) => f.id === highlightFileId);
    if (!file) return;

    // Expand every ancestor directory of this file.
    // Use untrack() when reading collapsedDirs so this effect only re-runs when
    // highlightFileId changes — not when collapsedDirs is written below (which
    // would otherwise cause an infinite loop).
    const parts = file.file_path.split("/");
    const next = untrack(() => new Set(collapsedDirs));
    for (let i = 1; i < parts.length; i++) {
      next.delete(parts.slice(0, i).join("/"));
    }
    collapsedDirs = next;

    // After Svelte flushes the DOM, scroll to the row and briefly highlight it
    tick().then(() => {
      const el = document.getElementById(`file-${highlightFileId}`);
      if (!el) return;
      el.scrollIntoView({ behavior: "smooth", block: "center" });
      el.style.transition = "background-color 0.25s";
      el.style.backgroundColor = "oklch(0.57 0.19 258 / 0.10)";
      el.style.borderRadius = "0.75rem";
      setTimeout(() => {
        el.style.backgroundColor = "";
      }, 1800);
    });
  });

  // ── Diff viewer state — lazy-loaded per file ──────────────────────────────────
  let expandedFiles = $state<Set<number>>(new Set());
  /** Cache: fileId → diff content string (loaded on first expand) */
  let diffCache = $state<Map<number, string>>(new Map());
  let loadingDiffs = $state<Set<number>>(new Set());

  async function toggleFile(file: PullHistoryFile) {
    const id = file.id;
    const next = new Set(expandedFiles);
    if (next.has(id)) {
      next.delete(id);
      expandedFiles = next;
      return;
    }
    next.add(id);
    expandedFiles = next;

    // Lazy-load diff content on first expand
    if (
      !diffCache.has(id) &&
      file.change_type !== "binary" &&
      file.change_type !== "renamed"
    ) {
      loadingDiffs = new Set(loadingDiffs).add(id);
      const content = await loadFileDiff(id);
      diffCache = new Map(diffCache).set(id, content);
      loadingDiffs = new Set([...loadingDiffs].filter((x) => x !== id));
    }
  }

  // ── Helpers ───────────────────────────────────────────────────────────────────
  const BADGE: Record<string, { label: string; cls: string; dot: string }> = {
    added: {
      label: "Added",
      cls: "bg-emerald-50 text-emerald-700 border-emerald-200/70",
      dot: "bg-emerald-500",
    },
    modified: {
      label: "Modified",
      cls: "bg-primary/8 text-primary border-primary/20",
      dot: "bg-primary",
    },
    deleted: {
      label: "Deleted",
      cls: "bg-red-50 text-red-700 border-red-200/70",
      dot: "bg-red-500",
    },
    renamed: {
      label: "Renamed",
      cls: "bg-amber-50 text-amber-700 border-amber-200/70",
      dot: "bg-amber-500",
    },
    binary: {
      label: "Binary",
      cls: "bg-muted text-muted-foreground border-border",
      dot: "bg-muted-foreground",
    },
  };

  function badge(ct: string) {
    return BADGE[ct] ?? BADGE.modified;
  }

  // Icon colour by change type
  const FILE_ICON_CLS: Record<string, string> = {
    added: "text-emerald-500",
    modified: "text-primary",
    deleted: "text-red-400",
    renamed: "text-amber-500",
    binary: "text-muted-foreground",
  };
</script>

<!-- Outer white card matching MASTER inner-cell pattern -->
<div class="bg-white/80 rounded-2xl border border-slate-200/70 overflow-hidden">
  <!-- Header -->
  <div
    class="px-4 py-2.5 border-b border-slate-100 flex items-center justify-between"
  >
    <span
      class="text-[11px] font-black uppercase tracking-widest text-muted-foreground"
    >
      Changed Files
      <span
        class="font-normal normal-case tracking-normal text-muted-foreground/70"
      >
        ({files.length})
      </span>
    </span>
    <div class="flex items-center gap-1.5">
      {#if totalAdditions > 0}
        <span
          class="text-[11px] font-bold px-1.5 py-0.5 rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200/70"
        >
          +{totalAdditions}
        </span>
      {/if}
      {#if totalDeletions > 0}
        <span
          class="text-[11px] font-bold px-1.5 py-0.5 rounded-full bg-red-50 text-red-700 border border-red-200/70"
        >
          -{totalDeletions}
        </span>
      {/if}
    </div>
  </div>

  <!-- Tree -->
  <div class="p-2 flex flex-col gap-px">
    {#each tree as node (node.path)}
      {@render treeNode(node)}
    {/each}
  </div>
</div>

<!-- ── Recursive tree snippet ───────────────────────────────────────────────── -->
{#snippet treeNode(node: TreeNode)}
  {#if node.type === "dir"}
    {@const collapsed = collapsedDirs.has(node.path)}

    <!-- Directory row -->
    <button
      type="button"
      onclick={() => toggleDir(node.path)}
      aria-expanded={!collapsed}
      class="w-full flex items-center gap-2 px-3 py-2 rounded-xl cursor-pointer text-left
             transition-all duration-140 border
             {collapsed
        ? 'bg-slate-50 border-slate-200/60 hover:bg-white hover:border-slate-200'
        : 'bg-accent/60 border-primary/15 hover:bg-accent'}"
    >
      <ChevronRight
        size={12}
        strokeWidth={2.5}
        class="shrink-0 text-muted-foreground transition-transform duration-140 {collapsed
          ? ''
          : 'rotate-90'}"
      />

      {#if collapsed}
        <Folder size={13} strokeWidth={2} class="shrink-0 text-primary" />
      {:else}
        <FolderOpen size={13} strokeWidth={2} class="shrink-0 text-primary" />
      {/if}

      <span
        class="flex-1 font-mono text-[12px] font-semibold min-w-0 truncate text-foreground"
      >
        {node.name}/
      </span>

      <!-- File count pill -->
      <span
        class="shrink-0 text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground border border-border"
      >
        {node.fileCount}
        {node.fileCount === 1 ? "file" : "files"}
      </span>

      <!-- Stat badges -->
      {#if node.additions > 0}
        <span
          class="shrink-0 text-[11px] font-bold px-1.5 py-0.5 rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200/70"
        >
          +{node.additions}
        </span>
      {/if}
      {#if node.deletions > 0}
        <span
          class="shrink-0 text-[11px] font-bold px-1.5 py-0.5 rounded-full bg-red-50 text-red-700 border border-red-200/70"
        >
          -{node.deletions}
        </span>
      {/if}
    </button>

    <!-- Children — indented with tree line -->
    {#if !collapsed}
      <div
        class="ml-5 pl-3 border-l-2 border-slate-200 flex flex-col gap-px my-px"
      >
        {#each node.children as child (child.path)}
          {@render treeNode(child)}
        {/each}
      </div>
    {/if}
  {:else}
    <!-- File row -->
    <div id="file-{node.file.id}">
      <button
        type="button"
        onclick={() => toggleFile(node.file)}
        aria-expanded={expandedFiles.has(node.file.id)}
        class="w-full flex items-center gap-2 px-3 py-2 rounded-xl cursor-pointer text-left
               transition-colors border border-transparent hover:bg-slate-50 hover:border-slate-200/60"
      >
        <!-- File icon coloured by change type -->
        <FileText
          size={12}
          strokeWidth={2.5}
          class="shrink-0 {FILE_ICON_CLS[node.file.change_type] ??
            'text-muted-foreground'}"
        />

        <!-- Filename -->
        {#if node.file.change_type === "renamed"}
          <span
            class="flex-1 font-mono text-[11.5px] min-w-0 truncate text-muted-foreground"
          >
            {node.file.file_path.split(" → ")[0]?.split("/").at(-1) ??
              node.name}
            <span class="mx-1 text-muted-foreground/50">→</span>
            <span class="text-foreground">
              {node.file.file_path.split(" → ")[1]?.split("/").at(-1) ?? ""}
            </span>
          </span>
        {:else}
          <span
            class="flex-1 font-mono text-[12px] min-w-0 truncate text-foreground"
            title={node.file.file_path}
          >
            {node.name}
          </span>
        {/if}

        <!-- Change type badge -->
        <span
          class="shrink-0 inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-semibold border {badge(
            node.file.change_type,
          ).cls}"
        >
          <span class="w-1 h-1 rounded-full {badge(node.file.change_type).dot}"
          ></span>
          {badge(node.file.change_type).label}
        </span>

        <!-- Stat badges -->
        {#if node.file.change_type !== "binary" && node.file.change_type !== "deleted"}
          {#if node.additions > 0}
            <span
              class="shrink-0 text-[11px] font-bold px-1.5 py-0.5 rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200/70"
            >
              +{node.additions}
            </span>
          {/if}
        {/if}
        {#if node.file.change_type !== "binary" && node.file.change_type !== "added"}
          {#if node.deletions > 0}
            <span
              class="shrink-0 text-[11px] font-bold px-1.5 py-0.5 rounded-full bg-red-50 text-red-700 border border-red-200/70"
            >
              -{node.deletions}
            </span>
          {/if}
        {/if}

        <!-- Diff expand indicator -->
        {#if node.file.change_type !== "binary" && node.file.change_type !== "renamed"}
          {#if loadingDiffs.has(node.file.id)}
            <Loader2
              size={11}
              strokeWidth={2.5}
              class="shrink-0 text-muted-foreground animate-spin"
            />
          {:else}
            <ChevronRight
              size={11}
              strokeWidth={2.5}
              class="shrink-0 text-muted-foreground transition-transform duration-140 {expandedFiles.has(
                node.file.id,
              )
                ? 'rotate-90'
                : ''}"
            />
          {/if}
        {/if}
      </button>

      <!-- Inline diff viewer — renders once diff is loaded -->
      {#if expandedFiles.has(node.file.id) && node.file.change_type !== "binary"}
        <div class="px-2 pb-2 pt-1">
          {#if diffCache.has(node.file.id)}
            <FileDiffViewer diffContent={diffCache.get(node.file.id)!} />
          {:else}
            <div
              class="flex items-center gap-2 px-3 py-2 text-[12px] text-muted-foreground"
            >
              <Loader2 size={13} class="animate-spin" /> Loading diff…
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
{/snippet}
