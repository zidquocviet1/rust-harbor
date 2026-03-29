<script lang="ts">
  import AiSettings from "./components/AiSettings.svelte";
  import GeneralSettings from "./components/GeneralSettings.svelte";
  import GitSettings from "./components/GitSettings.svelte";
  import PerformanceSettings from "./components/PerformanceSettings.svelte";
  import SecuritySettings from "./components/SecuritySettings.svelte";
  import SettingsSidebar from "./components/SettingsSidebar.svelte";

  let activeTab = $state<"general" | "performance" | "security" | "git" | "ai">(
    "general",
  );
</script>

<div
  class="p-8 space-y-8 animate-in fade-in duration-500 max-w-5xl mx-auto min-h-screen"
>
  <div class="space-y-2">
    <h2 class="text-4xl font-black tracking-tight text-glow uppercase">
      System Core
    </h2>
    <p class="text-muted-foreground text-lg opacity-70">
      Orchestrate your workspace and execution environment.
    </p>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-12 pt-4">
    <!-- Left: Navigation -->
    <SettingsSidebar {activeTab} onTabChange={(tab) => (activeTab = tab)} />

    <!-- Right: Content Area -->
    <div class="md:col-span-2">
      {#if activeTab === "general"}
        <div class="animate-in fade-in slide-in-from-right-4 duration-500">
          <GeneralSettings />
        </div>
      {:else if activeTab === "performance"}
        <div class="animate-in fade-in slide-in-from-right-4 duration-500">
          <PerformanceSettings />
        </div>
      {:else if activeTab === "git"}
        <div class="animate-in fade-in slide-in-from-right-4 duration-500">
          <GitSettings />
        </div>
      {:else if activeTab === "security"}
        <div class="animate-in fade-in slide-in-from-right-4 duration-500">
          <SecuritySettings />
        </div>
      {:else if activeTab === "ai"}
        <div class="animate-in fade-in slide-in-from-right-4 duration-500">
          <AiSettings />
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .settings-container {
    display: block;
  }
</style>
