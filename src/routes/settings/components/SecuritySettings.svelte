<script lang="ts">
  import { Shield, ShieldCheck, Lock, EyeOff } from "lucide-svelte";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card";
  import { APP_BRANDING } from "$lib/config/branding";
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
  <Card class="glass border-white/5 opacity-80 filter grayscale-[0.5]">
    <CardHeader>
      <div class="flex items-center space-x-3">
        <div class="p-2 rounded-lg bg-amber-500/10 text-amber-500">
          <Shield class="w-5 h-5" />
        </div>
        <div>
          <CardTitle class="text-lg">Security & Privacy</CardTitle>
          <CardDescription>Localized data protection protocols.</CardDescription>
        </div>
      </div>
    </CardHeader>
    <CardContent class="space-y-6">
       <div class="space-y-4">
          <div class="p-4 rounded-2xl bg-amber-500/5 border border-amber-500/10 space-y-3">
             <div class="flex items-center gap-2 text-amber-500">
                <ShieldCheck class="w-4 h-4" />
                <span class="text-xs font-bold uppercase tracking-wider">Privacy Guaranteed</span>
             </div>
             <p class="text-xs text-amber-200/50 leading-relaxed">
               {APP_BRANDING.shortName} operates strictly on your local machine. We do not upload your source code, configuration, or environment metrics to any external server.
             </p>
          </div>

          <div class="grid grid-cols-2 gap-4">
             <div class="p-4 rounded-2xl border border-white/5 bg-white/[0.01] space-y-2 opacity-50 cursor-not-allowed">
                <Lock class="w-4 h-4 text-muted-foreground" />
                <p class="text-xs font-semibold">Keychain Sync</p>
                <p class="text-[10px] text-muted-foreground">Secure credential storage for remotes.</p>
             </div>
             <div class="p-4 rounded-2xl border border-white/5 bg-white/[0.01] space-y-2 opacity-50 cursor-not-allowed">
                <EyeOff class="w-4 h-4 text-muted-foreground" />
                <p class="text-xs font-semibold">Folder Cloaking</p>
                <p class="text-[10px] text-muted-foreground">Hide sensitive paths from global search.</p>
             </div>
          </div>
       </div>
       
       <div class="text-center py-2">
          <span class="text-[10px] font-bold text-primary/40 uppercase tracking-[0.2em]">Coming Soon</span>
       </div>
    </CardContent>
  </Card>
</div>
