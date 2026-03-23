<script lang="ts">
  import { goto } from "$app/navigation";
  import { GitPullRequest, Home } from "lucide-svelte";
</script>

<div
  class="flex flex-col items-center justify-center py-20 px-8 text-center gap-4"
>
  <div
    class="w-20 h-20 rounded-[var(--radius-xl)] flex items-center justify-center mb-1 bg-primary/8"
  >
    <GitPullRequest size={40} strokeWidth={1.5} class="text-primary/45" />
  </div>

  <h3 class="text-[18px] font-bold text-foreground">No pull history yet</h3>

  <p class="text-[13.5px] leading-relaxed max-w-xs text-muted-foreground">
    Pull a repository to see file changes here.<br />
    History is saved permanently, even after you remove a repo.
  </p>

  <button
    type="button"
    onclick={() => goto("/")}
    class="mt-2 flex items-center gap-2 px-4 py-2 rounded-[var(--radius-sm)] text-[13px] font-semibold text-primary-foreground cursor-pointer transition-colors bg-primary hover:bg-primary/90"
  >
    <Home size={13} strokeWidth={2.5} />
    Go to Repositories
  </button>
</div>
