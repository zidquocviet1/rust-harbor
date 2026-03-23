<script lang="ts">
  interface Props {
    diffContent: string;
  }

  let { diffContent }: Props = $props();

  interface DiffLine {
    type: "add" | "del" | "ctx" | "hunk" | "meta";
    before: number | null;
    after: number | null;
    content: string;
  }

  const parsed = $derived(parseDiff(diffContent));

  function parseDiff(raw: string): DiffLine[] {
    if (!raw || raw === "[binary file]") {
      return [{ type: "meta", before: null, after: null, content: raw || "" }];
    }

    const lines: DiffLine[] = [];
    let beforeLine = 0;
    let afterLine = 0;

    for (const line of raw.split("\n")) {
      if (line.startsWith("@@")) {
        // Parse hunk header: @@ -X,Y +A,B @@
        const m = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
        if (m) {
          beforeLine = parseInt(m[1], 10);
          afterLine = parseInt(m[2], 10);
        }
        lines.push({ type: "hunk", before: null, after: null, content: line });
      } else if (line.startsWith("+")) {
        lines.push({
          type: "add",
          before: null,
          after: afterLine++,
          content: line.slice(1),
        });
      } else if (line.startsWith("-")) {
        lines.push({
          type: "del",
          before: beforeLine++,
          after: null,
          content: line.slice(1),
        });
      } else if (
        line.startsWith("diff --git") ||
        line.startsWith("index ") ||
        line.startsWith("--- ") ||
        line.startsWith("+++ ") ||
        line.startsWith("new file") ||
        line.startsWith("deleted file") ||
        line.startsWith("rename") ||
        line.startsWith("similarity")
      ) {
        lines.push({ type: "meta", before: null, after: null, content: line });
      } else {
        const c = line.startsWith(" ") ? line.slice(1) : line;
        lines.push({
          type: "ctx",
          before: beforeLine++,
          after: afterLine++,
          content: c,
        });
      }
    }
    return lines;
  }
</script>

<style>
  .diff-scroll {
    scrollbar-width: thin;
    scrollbar-color: oklch(0.35 0.02 260) oklch(0.17 0.015 260);
  }
  .diff-scroll::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  .diff-scroll::-webkit-scrollbar-track {
    background: oklch(0.17 0.015 260);
  }
  .diff-scroll::-webkit-scrollbar-thumb {
    background: oklch(0.35 0.02 260);
    border-radius: 4px;
    border: 2px solid oklch(0.17 0.015 260);
  }
  .diff-scroll::-webkit-scrollbar-thumb:hover {
    background: oklch(0.45 0.03 260);
  }
  .diff-scroll::-webkit-scrollbar-corner {
    background: oklch(0.17 0.015 260);
  }
</style>

<div
  class="diff-viewer overflow-hidden rounded-[var(--radius-sm)] border border-slate-700/80"
  style="background: oklch(0.14 0.015 260);"
>
  {#if diffContent === "[binary file]"}
    <div
      class="px-4 py-3 text-xs font-mono"
      style="color: oklch(0.62 0.01 250);"
    >
      Binary file — diff not available
    </div>
  {:else if diffContent.includes("[diff truncated]")}
    <div
      class="px-3 py-1.5 text-xs border-b"
      style="color:oklch(0.72 0.12 75); background: oklch(0.18 0.025 75 / 0.3); border-color: oklch(0.22 0.02 260);"
    >
      Diff truncated at 500 KB
    </div>
  {/if}

  <div class="diff-scroll overscroll-contain" style="max-height: 420px; overflow: auto;">
    <table
      class="w-full border-collapse"
      style="font-family: var(--font-mono, 'JetBrains Mono', monospace); font-size: 12px;"
    >
      <tbody>
        {#each parsed as line (line)}
          {#if line.type === "meta"}
            <!-- skip meta lines (diff --git, index, ---, +++) -->
          {:else if line.type === "hunk"}
            <tr>
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.17 0.018 260); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.40 0.01 250); font-size:11px;"
              ></td>
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.17 0.018 260); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.40 0.01 250); font-size:11px;"
              ></td>
              <td
                class="px-3 py-0.5 whitespace-pre"
                style="background: oklch(0.21 0.04 258 / 0.5); color: oklch(0.60 0.12 258);"
                >{line.content}</td
              >
            </tr>
          {:else if line.type === "add"}
            <tr>
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.20 0.04 145); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.55 0.14 145); font-size:11px;"
              ></td>
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.20 0.04 145); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.60 0.15 145); font-size:11px;"
                >{line.after}</td
              >
              <td
                class="px-3 py-0.5 whitespace-pre"
                style="background: oklch(0.19 0.04 145 / 0.6); color: oklch(0.82 0.10 145);"
                ><span style="color: oklch(0.60 0.18 145); user-select:none;"
                  >+</span
                >{line.content}</td
              >
            </tr>
          {:else if line.type === "del"}
            <tr>
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.18 0.04 24); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.60 0.18 24); font-size:11px;"
                >{line.before}</td
              >
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.18 0.04 24); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.40 0.01 250); font-size:11px;"
              ></td>
              <td
                class="px-3 py-0.5 whitespace-pre"
                style="background: oklch(0.17 0.04 24 / 0.6); color: oklch(0.80 0.12 24);"
                ><span style="color: oklch(0.60 0.22 24); user-select:none;"
                  >-</span
                >{line.content}</td
              >
            </tr>
          {:else}
            <tr>
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.15 0.015 260); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.40 0.01 250); font-size:11px;"
                >{line.before ?? ""}</td
              >
              <td
                class="ln"
                style="width:42px; padding: 1px 8px; text-align:right; user-select:none; background: oklch(0.15 0.015 260); border-right: 1px solid oklch(0.22 0.02 260); color: oklch(0.40 0.01 250); font-size:11px;"
                >{line.after ?? ""}</td
              >
              <td
                class="px-3 py-0.5 whitespace-pre"
                style="color: oklch(0.62 0.01 250);"
                ><span style="opacity:0; user-select:none;">
                </span>{line.content}</td
              >
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  </div>
</div>
