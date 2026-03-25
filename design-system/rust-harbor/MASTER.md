# Design System Master — Rust Harbor

> **LOGIC:** When building a specific page, check `design-system/pages/[page-name].md` first.
> If that file exists, its rules **override** this Master. If not, strictly follow the rules below.

---

**Project:** Rust Harbor
**Updated:** 2026-03-23
**Type:** Tauri desktop app — Git repository manager
**Style:** Glassmorphism / soft-light — professional dev tool

---

## Color Palette

All colors use **OKLCH**. Use Tailwind semantic tokens (`text-primary`, `bg-muted`, etc.) wherever possible. Raw OKLCH is only for `style=` attributes.

| Role | OKLCH | Tailwind token | Notes |
|------|-------|---------------|-------|
| Background | `oklch(0.985 0.004 220)` | `bg-background` | Off-white, subtle blue tint |
| Foreground | `oklch(0.23 0.02 250)` | `text-foreground` | Near-black |
| Primary | `oklch(0.57 0.19 258)` | `text-primary / bg-primary` | Indigo — main brand colour |
| Primary fg | `oklch(0.99 0.003 220)` | `text-primary-foreground` | Near-white on primary |
| Muted bg | `oklch(0.95 0.008 230)` | `bg-muted` | Subtle section bg |
| Muted fg | `oklch(0.48 0.02 250)` | `text-muted-foreground` | Secondary labels |
| Accent bg | `oklch(0.95 0.014 252)` | `bg-accent` | Light indigo tint |
| Destructive | `oklch(0.62 0.22 24)` | `text-destructive / bg-destructive` | Red — delete/danger |
| Border | `oklch(0.86 0.01 240)` | `border-border` | Light grey lines |
| Ring | `oklch(0.62 0.16 255)` | `ring-ring` | Focus ring (indigo) |

### Opacity variants in use
```
bg-primary/4    bg-primary/8    bg-primary/10   bg-primary/12
bg-primary/20   border-primary/20  border-primary/30  border-primary/40
text-primary/80  shadow-primary/20
```

### Named surfaces (use in style= when no Tailwind token exists)
```
--surface-2:        oklch(1 0 0 / 0.72)          glass fill base
--surface-overlay:  oklch(0.99 0.004 220 / 0.94)  toasts, popovers
```

---

## Typography

| Role | Family | Weights | CSS var |
|------|--------|---------|---------|
| Body / UI | DM Sans | 400 500 600 700 | `--font-sans` / `font-sans` |
| Headings / Display | Space Grotesk | 500 600 700 | `--font-display` |
| Code / Mono | JetBrains Mono | 400 500 600 | `--font-mono` / `font-mono` |

**Base:** 16px body, 1.55 line-height, font-feature-settings cv02/cv03/cv04/cv11.

### Heading scale (all use Space Grotesk via `h1–h4` base rule)
| Element | Classes |
|---------|---------|
| Page title (h1) | `text-5xl font-black tracking-tighter text-glow` |
| Page subtitle | `text-xl text-muted-foreground` + `text-primary font-bold` for highlight |
| Section label (kicker) | `text-[10px] font-black uppercase tracking-widest text-muted-foreground` |
| Card title | `text-2xl font-black tracking-tight` |
| Sidebar title | `sidebar-title` class = Space Grotesk 18px |

---

## Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | `0.6rem` (~10px) | Small inner elements |
| `--radius-md` | `0.725rem` (~12px) | Inputs, tags |
| `--radius-lg` | `0.85rem` (~14px) | Default rounded-xl cards |
| `--radius-xl` | `1.1rem` (~18px) | Larger modals |
| `rounded-2xl` | `1rem` (16px) | Inner stat cells in repo cards |
| `rounded-[1.5rem]` | `24px` | Main repo / history cards outer |

---

## Motion

| Token | Value | Usage |
|-------|-------|-------|
| `--motion-fast` | `140ms` | Micro-interactions, chevrons |
| `--motion-std` | `220ms` | Card entrance, panel transitions |
| `--stagger-step` | `30ms` | List stagger delay per item |

Entrance animation: `animate-in fade-in duration-500` on page wrapper.
Card stagger: `.repo-card-enter` with `--stagger-index` CSS var (capped at 12).

---

## Global Background

Every page has this layered background (from +layout.svelte):
```svelte
<div class="fixed inset-0 bg-background -z-20"></div>
<div class="fixed inset-0 bg-gradient-to-tr from-primary/10 via-transparent to-sky-100/40 -z-10 pointer-events-none"></div>
<!-- Ambient glows -->
<div class="fixed top-[-10%] left-[-10%] w-[40%] h-[40%] bg-primary/12 rounded-full blur-[120px] -z-10 animate-pulse"></div>
<div class="fixed bottom-[-10%] right-[-10%] w-[30%] h-[30%] bg-cyan-400/10 rounded-full blur-[100px] -z-10 animate-pulse delay-700"></div>
```
Pages themselves do **not** add their own background — they sit on this global layer.

---

## Key Utility Classes

### `.glass`
```css
background: color-mix(in oklab, white 96%, var(--surface-2));
box-shadow: 0 1px 0 rgba(255,255,255,0.85) inset, 0 6px 16px rgba(15,23,42,0.05);
border: 1px solid oklch(0.86 0.01 240 / 0.85);
backdrop-filter: blur(12px);
```
**Use on:** cards, sidebar, popovers, panels.

### `.glass-hover`
```
hover:bg-white hover:border-slate-300/90 hover:shadow-[0_10px_22px_rgba(15,23,42,0.09)] hover:-translate-y-0.5 transition-all duration-200
```

### `.bg-grid-white`
Subtle 36×36px grid using `oklch(0.92 0.008 240 / 0.45)` lines. Use on hero / empty state areas.

---

## Sidebar

```
width:       w-72 (288px), sticky h-screen
style:       glass border-r border-slate-200/70
font:        sidebar-shell (DM Sans 14px)
padding:     p-4, space-y-2
```

### Nav item
```svelte
class="sidebar-item flex items-center space-x-3 px-4 py-3 rounded-2xl transition-all duration-200 group
  {active
    ? 'bg-primary/12 text-primary border border-primary/30 shadow-sm shadow-primary/20'
    : 'hover:bg-slate-100/80 text-muted-foreground border border-transparent hover:border-slate-200/80'}"
```
Icon size: **18px**. Badge (unread): `bg-primary text-white text-[10px] font-bold min-w-[18px] h-[18px] rounded-full px-1.5`.

---

## Page Layout

```svelte
<div class="p-8 space-y-8 animate-in fade-in duration-500 max-w-{SIZE} mx-auto min-h-screen">
```

| Page | max-w |
|------|-------|
| Repository List | `max-w-7xl` |
| Pull History | `max-w-5xl` |
| Settings | `max-w-5xl` |

### Page header pattern
```svelte
<h1 class="text-5xl font-black tracking-tighter text-glow mb-2">Page Title</h1>
<p class="text-muted-foreground text-xl">
  <span class="text-primary font-bold">{count}</span> items recorded
</p>
```

---

## Components

### Cards (main content cards)

```svelte
class="glass border-slate-200/70 shadow-none rounded-[1.5rem] overflow-hidden
       transition-[background-color,border-color,box-shadow,transform] duration-200
       hover:bg-white hover:border-slate-300/90 hover:shadow-[0_10px_22px_rgba(15,23,42,0.09)] hover:-translate-y-0.5
       repo-card-enter"
style="--stagger-index: {i}"
```

Inner stat/info cells:
```svelte
class="bg-white/80 rounded-2xl p-3 border border-slate-200/70 hover:border-slate-200/80 transition-all"
```

### Inputs / Search

```svelte
class="bg-white/80 border border-slate-200/80 rounded-xl px-4 py-2 text-sm
       focus:outline-none focus:ring-2 focus:ring-primary/40 focus:bg-white transition-all font-medium"
```
With icon on left: add `pl-10` and position icon `absolute left-3 top-1/2 -translate-y-1/2`.

### Toolbar button groups

```svelte
<!-- Wrapper -->
class="bg-white/80 rounded-xl border border-slate-200/80 p-1 flex items-center"
<!-- Individual button -->
class="p-2 rounded-lg transition-all {active ? 'bg-white/10 text-primary shadow-inner' : 'text-muted-foreground hover:bg-slate-100'}"
```

### Standalone filter/action buttons

```svelte
class="flex items-center gap-2 bg-white/80 border border-slate-200/80 rounded-xl px-3 py-2
       text-sm font-medium cursor-pointer transition-all hover:bg-white hover:border-slate-300/80
       focus:outline-none focus:ring-2 focus:ring-primary/40
       {active ? 'border-primary/40 text-foreground' : 'text-muted-foreground'}"
```

### Filter chips (language / tag filters)

```svelte
<!-- Container -->
class="bg-white/80 rounded-xl border border-slate-200/80 p-1.5 flex items-center gap-1.5"
<!-- Chip inactive -->
class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg
       hover:bg-slate-100 text-muted-foreground border border-transparent transition-all"
<!-- Chip active -->
class="px-3.5 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg
       bg-primary text-primary-foreground border-primary/20 shadow-inner transition-all"
```

### Badges / pills

```svelte
<!-- Language badge -->
class="bg-white/80 border border-slate-200/70 text-[8px] px-2 py-0.5 rounded-md font-black uppercase tracking-widest text-muted-foreground"
<!-- Tag badge (coloured dot) -->
class="bg-white/80 border text-[8px] px-2 py-0.5 rounded-full font-black uppercase tracking-widest flex items-center gap-1"
<!-- Count / status pill -->
class="text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground border border-border"
```

### Destructive button (outlined)

```svelte
class="flex items-center gap-1.5 px-3.5 py-2 rounded-xl border text-sm font-semibold cursor-pointer
       text-destructive border-destructive/35 bg-transparent hover:bg-destructive/8 transition-colors"
```

### Portalled dropdown

```svelte
<!-- Panel -->
class="fixed bg-background/98 border border-slate-200/80 rounded-2xl
       shadow-[0_20px_50px_rgba(0,0,0,0.5)] backdrop-blur-2xl z-[110] p-3
       animate-in fade-in zoom-in-95 slide-in-from-top-2 duration-200 ring-1 ring-slate-200/70"
<!-- Section header -->
class="px-3 py-2 mb-2 border-b border-slate-200/70 text-[9px] font-black uppercase tracking-[0.2em] text-muted-foreground"
<!-- Item inactive -->
class="w-full text-left px-3 py-2.5 text-[10px] font-black uppercase tracking-widest rounded-xl
       hover:bg-slate-100 text-muted-foreground transition-all"
<!-- Item active -->
class="bg-primary/10 text-primary"
```

### Glass floating toolbar (multi-select)

```svelte
class="fixed bottom-6 left-1/2 -translate-x-1/2 flex items-center gap-3 px-5 py-3
       rounded-[var(--radius-lg)] border z-50 glass"
style="box-shadow: 0 8px 40px oklch(0 0 0 / 0.12);"
```

---

## Icons

**Library:** `lucide-svelte` exclusively.
**Size conventions:**

| Context | Size |
|---------|------|
| Sidebar nav | 18px |
| Page action buttons | 13–14px |
| Card metadata | 12–14px |
| Badge / inline | 10–12px |

No emojis as icons. No mixing icon sets.

---

## Anti-Patterns

- ❌ Raw hex colours — use OKLCH or Tailwind tokens
- ❌ Dark background cards — this is a **light** theme app
- ❌ Emojis as icons
- ❌ Missing `cursor-pointer` on clickable elements
- ❌ Scale transforms on hover (layout shift) — use `translate-y` only
- ❌ Instant state changes — always 140–220ms transition
- ❌ Invisible focus states — `focus:ring-2 focus:ring-primary/40`
- ❌ `inline oklch()` for values that have a Tailwind token

---

## Pre-Delivery Checklist

- [ ] All cards use `.glass` + `rounded-[1.5rem]` for outer shell
- [ ] Page wrapped in `p-8 space-y-8 animate-in fade-in duration-500 max-w-{X} mx-auto min-h-screen`
- [ ] Page title: `text-5xl font-black tracking-tighter text-glow`
- [ ] All inputs: `bg-white/80 border border-slate-200/80 rounded-xl focus:ring-2 focus:ring-primary/40`
- [ ] Active state colour uses `bg-primary/12 text-primary border-primary/30`
- [ ] Hover lift: `hover:-translate-y-0.5` (not scale)
- [ ] Icons from lucide-svelte only
- [ ] `cursor-pointer` on all buttons/interactive elements
- [ ] Transitions 140–220ms on all interactive elements
- [ ] Focus states visible (`focus:ring-2 focus:ring-primary/40`)
