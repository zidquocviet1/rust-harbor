<script lang="ts">
  interface Props {
    provider: string;
    class?: string;
  }
  let { provider, class: className = "" }: Props = $props();
</script>

{#if provider === "claude"}
  <span
    class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-amber-500 text-white shrink-0 {className}"
  >
    <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
      <polygon points="5,1 9.3,8.5 0.7,8.5" />
    </svg>
  </span>
{:else if provider === "openai"}
  <span
    class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-slate-800 text-white shrink-0 {className}"
  >
    <svg
      width="10"
      height="10"
      viewBox="0 0 10 10"
      stroke="currentColor"
      stroke-width="1.4"
      stroke-linecap="round"
      fill="none"
    >
      <line x1="5" y1="1" x2="5" y2="9" />
      <line x1="1.3" y1="3" x2="8.7" y2="7" />
      <line x1="8.7" y1="3" x2="1.3" y2="7" />
    </svg>
  </span>
{:else if provider === "gemini"}
  <span
    class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-blue-500 text-white shrink-0 {className}"
  >
    <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
      <path
        d="M5 0.5 L6.2 3.8 L9.5 5 L6.2 6.2 L5 9.5 L3.8 6.2 L0.5 5 L3.8 3.8 Z"
      />
    </svg>
  </span>
{:else if provider === "grok"}
  <span
    class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-zinc-900 text-white shrink-0 {className}"
  >
    <svg
      width="10"
      height="10"
      viewBox="0 0 10 10"
      stroke="currentColor"
      stroke-width="2.2"
      stroke-linecap="round"
      fill="none"
    >
      <line x1="2" y1="2" x2="8" y2="8" />
      <line x1="8" y1="2" x2="2" y2="8" />
    </svg>
  </span>
{:else if provider === "ollama"}
  <span
    class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-emerald-500 text-white shrink-0 {className}"
  >
    <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
      <circle cx="5" cy="3.2" r="1.8" />
      <path d="M2.5 9 C2.5 6.5 3.5 5.5 5 5.5 C6.5 5.5 7.5 6.5 7.5 9 Z" />
    </svg>
  </span>
{:else}
  <span
    class="w-[18px] h-[18px] rounded-[4px] flex items-center justify-center bg-slate-400 text-white shrink-0 {className}"
  >
    <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
      <circle cx="5" cy="5" r="3.5" />
    </svg>
  </span>
{/if}
