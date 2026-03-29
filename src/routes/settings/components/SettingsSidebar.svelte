<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Binary, Bot, Database, Shield, Zap } from "lucide-svelte";

  interface Props {
    activeTab: "general" | "performance" | "security" | "git" | "ai";
    onTabChange: (
      tab: "general" | "performance" | "security" | "git" | "ai",
    ) => void;
  }

  let { activeTab, onTabChange }: Props = $props();

  const tabs = [
    { id: "general", label: "Watched Folders", icon: Database },
    { id: "performance", label: "Performance", icon: Zap },
    { id: "security", label: "Security", icon: Shield },
    { id: "git", label: "Git Binary", icon: Binary },
    { id: "ai", label: "AI Settings", icon: Bot },
  ] as const;
</script>

<div class="space-y-3">
  {#each tabs as tab}
    <Button
      variant={activeTab === tab.id ? "secondary" : "ghost"}
      class="w-full justify-start rounded-xl py-6 transition-all duration-300 {activeTab ===
      tab.id
        ? 'bg-primary/10 text-primary border border-primary/20 shadow-lg shadow-primary/5'
        : 'hover:bg-white/5 opacity-70 hover:opacity-100'}"
      onclick={() => onTabChange(tab.id)}
    >
      <tab.icon class="w-4 h-4 mr-3" />
      {tab.label}
    </Button>
  {/each}
</div>
