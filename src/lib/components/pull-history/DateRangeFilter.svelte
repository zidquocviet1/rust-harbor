<script lang="ts">
  import { dateFrom, dateTo } from "$lib/stores/pullHistoryStore";
  import { Popover, RangeCalendar } from "bits-ui";
  import { CalendarDays, ChevronDown, Check, X, ChevronLeft, ChevronRight } from "lucide-svelte";
  import { today, getLocalTimeZone, CalendarDate } from "@internationalized/date";
  import type { DateRange, DateValue } from "@internationalized/date";

  type Mode = "all" | "last24h" | "last7d" | "last30d" | "last3m" | "custom";

  const PRESETS: { id: Exclude<Mode, "custom">; label: string; hint: string }[] = [
    { id: "all",     label: "Any time",      hint: "No date filter" },
    { id: "last24h", label: "Last 24 hours", hint: "Since yesterday" },
    { id: "last7d",  label: "Last 7 days",   hint: "Past week" },
    { id: "last30d", label: "Last 30 days",  hint: "Past month" },
    { id: "last3m",  label: "Last 3 months", hint: "Past quarter" },
  ];

  // Initialise from stores so navigating away and back restores the correct mode.
  let mode = $state<Mode>($dateFrom !== null || $dateTo !== null ? "custom" : "all");
  let open = $state(false);

  const todayDate = today(getLocalTimeZone());
  const tz = getLocalTimeZone();

  // Sync mode → "all" when store is cleared externally
  $effect(() => {
    if ($dateFrom === null && $dateTo === null && mode !== "custom") {
      mode = "all";
    }
  });

  function toCalendarDate(d: Date | null): DateValue | undefined {
    if (!d) return undefined;
    return new CalendarDate(d.getFullYear(), d.getMonth() + 1, d.getDate());
  }

  const rangeValue = $derived<DateRange>({
    start: toCalendarDate($dateFrom),
    end: toCalendarDate($dateTo),
  });

  function applyPreset(id: Exclude<Mode, "custom">) {
    mode = id;
    if (id === "all") {
      dateFrom.set(null);
      dateTo.set(null);
      open = false;
      return;
    }
    const now  = new Date();
    const from = new Date();
    if      (id === "last24h") from.setHours(from.getHours() - 24);
    else if (id === "last7d")  from.setDate(from.getDate() - 7);
    else if (id === "last30d") from.setDate(from.getDate() - 30);
    else if (id === "last3m")  from.setMonth(from.getMonth() - 3);
    dateFrom.set(from);
    dateTo.set(now);
    open = false;
  }

  function selectCustom() {
    mode = "custom";
  }

  function handleRangeChange(range: DateRange) {
    if (range.start) {
      dateFrom.set(range.start.toDate(tz));
    } else {
      dateFrom.set(null);
    }
    if (range.end) {
      const end = range.end.toDate(tz);
      end.setHours(23, 59, 59, 999);
      dateTo.set(end);
      // Both selected → close the popover
      open = false;
    } else {
      dateTo.set(null);
    }
  }

  function clearFilter() {
    mode = "all";
    dateFrom.set(null);
    dateTo.set(null);
    open = false;
  }

  const isActive = $derived(mode !== "all");

  const triggerLabel = $derived.by(() => {
    if (mode === "all")     return "Any time";
    if (mode === "last24h") return "Last 24 hours";
    if (mode === "last7d")  return "Last 7 days";
    if (mode === "last30d") return "Last 30 days";
    if (mode === "last3m")  return "Last 3 months";
    const from = $dateFrom;
    const to   = $dateTo;
    const fmt  = (d: Date) =>
      d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
    if (from && to)  return `${fmt(from)} – ${fmt(to)}`;
    if (from)        return `From ${fmt(from)}`;
    if (to)          return `Until ${fmt(to)}`;
    return "Custom range";
  });

  function formatMonthHeading(dv: DateValue): string {
    return dv.toDate(tz).toLocaleDateString("en-US", { month: "long", year: "numeric" });
  }
</script>

<div class="flex items-center gap-1">
  <Popover.Root bind:open>
    <!-- Trigger -->
    <Popover.Trigger
      class="flex items-center gap-2 bg-white/80 border rounded-xl pl-3 pr-2.5 py-2 text-sm font-medium cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/40 transition-all hover:bg-white {isActive
        ? 'border-primary/40 text-foreground hover:border-primary/60'
        : 'border-slate-200/80 text-muted-foreground hover:border-slate-300/80'}"
    >
      <CalendarDays
        size={13}
        class="shrink-0 {isActive ? 'text-primary' : 'text-muted-foreground'}"
      />
      <span>{triggerLabel}</span>
      <ChevronDown
        size={12}
        class="shrink-0 text-muted-foreground transition-transform duration-150 {open ? 'rotate-180' : ''}"
      />
    </Popover.Trigger>

    <!-- Dropdown panel -->
    <Popover.Content
      class="z-50 bg-white border border-slate-200 rounded-xl shadow-2xl py-2 outline-none {mode === 'custom' ? 'w-auto' : 'w-60'}"
      sideOffset={6}
      align="start"
    >
      <!-- Section header -->
      <div class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400">
        Time Period
      </div>

      <!-- Preset rows -->
      {#each PRESETS as preset (preset.id)}
        <button
          type="button"
          class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-100 text-left transition-colors group"
          onclick={() => applyPreset(preset.id)}
        >
          <div class="w-5 h-5 flex items-center justify-center shrink-0">
            {#if mode === preset.id}
              <Check size={13} class="text-primary" />
            {/if}
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium transition-colors {mode === preset.id
              ? 'text-primary'
              : 'text-slate-700 group-hover:text-primary'}">
              {preset.label}
            </div>
            {#if preset.id !== "all"}
              <div class="text-[11px] text-slate-400">{preset.hint}</div>
            {/if}
          </div>
        </button>
      {/each}

      <!-- Divider -->
      <div class="mx-3 my-1 border-t border-slate-100"></div>

      <!-- Custom range row -->
      <button
        type="button"
        class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-slate-100 text-left transition-colors group"
        onclick={selectCustom}
      >
        <div class="w-5 h-5 flex items-center justify-center shrink-0">
          {#if mode === "custom"}
            <Check size={13} class="text-primary" />
          {/if}
        </div>
        <span class="flex-1 text-sm font-medium transition-colors {mode === 'custom'
          ? 'text-primary'
          : 'text-slate-700 group-hover:text-primary'}">
          Custom range
        </span>
      </button>

      <!-- Inline range calendar when custom is selected -->
      {#if mode === "custom"}
        <div class="border-t border-slate-100 mt-1 px-3 pt-3 pb-2">
          <RangeCalendar.Root
            value={rangeValue}
            onValueChange={handleRangeChange}
            maxValue={todayDate}
            numberOfMonths={2}
            pagedNavigation={true}
            weekdayFormat="short"
            class="range-cal select-none"
          >
            {#snippet children({ months, weekdays })}
              <div class="flex gap-5">
                {#each months as month, i}
                  <div class="min-w-[216px]">
                    <!-- Month header -->
                    <div class="relative flex items-center justify-center mb-3 h-7">
                      {#if i === 0}
                        <RangeCalendar.PrevButton
                          class="absolute left-0 inline-flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-slate-100 transition-colors focus:outline-none disabled:opacity-40 cursor-pointer"
                        >
                          <ChevronLeft size={14} />
                        </RangeCalendar.PrevButton>
                      {/if}

                      <span class="text-sm font-semibold text-foreground">
                        {formatMonthHeading(month.value)}
                      </span>

                      {#if i === months.length - 1}
                        <RangeCalendar.NextButton
                          class="absolute right-0 inline-flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-slate-100 transition-colors focus:outline-none disabled:opacity-40 cursor-pointer"
                        >
                          <ChevronRight size={14} />
                        </RangeCalendar.NextButton>
                      {/if}
                    </div>

                    <!-- Day grid -->
                    <RangeCalendar.Grid class="w-full border-collapse">
                      <RangeCalendar.GridHead>
                        <RangeCalendar.GridRow class="flex mb-1">
                          {#each weekdays as weekday}
                            <RangeCalendar.HeadCell
                              class="w-[30px] h-6 flex items-center justify-center text-[10px] font-medium text-muted-foreground"
                            >
                              {weekday}
                            </RangeCalendar.HeadCell>
                          {/each}
                        </RangeCalendar.GridRow>
                      </RangeCalendar.GridHead>

                      <RangeCalendar.GridBody>
                        {#each month.weeks as week}
                          <RangeCalendar.GridRow class="flex">
                            {#each week as date}
                              <RangeCalendar.Cell
                                {date}
                                month={month.value}
                                class="rc-cell relative p-0 w-[30px] h-[30px]"
                              >
                                <RangeCalendar.Day
                                  class="rc-day inline-flex h-[30px] w-[30px] items-center justify-center rounded-full text-[13px] transition-colors cursor-pointer focus:outline-none
                                    hover:bg-slate-100
                                    data-[selection-start]:bg-primary data-[selection-start]:text-primary-foreground data-[selection-start]:hover:bg-primary/90 data-[selection-start]:rounded-full
                                    data-[selection-end]:bg-primary data-[selection-end]:text-primary-foreground data-[selection-end]:hover:bg-primary/90 data-[selection-end]:rounded-full
                                    data-[range-middle]:bg-transparent data-[range-middle]:rounded-none data-[range-middle]:hover:bg-primary/20
                                    data-[highlighted]:bg-primary/15 data-[highlighted]:rounded-none
                                    data-[today]:font-bold data-[today]:not-data-[selected]:text-primary
                                    data-[outside-month]:text-muted-foreground/25 data-[outside-month]:pointer-events-none
                                    data-[disabled]:opacity-30 data-[disabled]:pointer-events-none data-[disabled]:cursor-default"
                                />
                              </RangeCalendar.Cell>
                            {/each}
                          </RangeCalendar.GridRow>
                        {/each}
                      </RangeCalendar.GridBody>
                    </RangeCalendar.Grid>
                  </div>
                {/each}
              </div>

              <!-- Selection hint -->
              {#if rangeValue.start && !rangeValue.end}
                <p class="text-center text-[11px] text-muted-foreground mt-2">
                  Now pick an end date
                </p>
              {:else if !rangeValue.start}
                <p class="text-center text-[11px] text-muted-foreground mt-2">
                  Pick a start date
                </p>
              {/if}
            {/snippet}
          </RangeCalendar.Root>
        </div>
      {/if}
    </Popover.Content>
  </Popover.Root>

  <!-- Clear button (outside trigger to avoid button-in-button) -->
  {#if isActive}
    <button
      type="button"
      onclick={clearFilter}
      class="inline-flex items-center justify-center w-6 h-6 rounded-lg text-muted-foreground hover:text-destructive hover:bg-destructive/8 transition-colors focus:outline-none"
      aria-label="Clear date filter"
    >
      <X size={12} />
    </button>
  {/if}
</div>

<style>
  /* --- Range path: fully :global so Svelte scope hash doesn't block them --- */

  /* Middle cells: continuous strip */
  :global(.rc-cell[data-range-middle]) {
    background-color: oklch(0.57 0.19 258 / 0.08);
  }
  /* Start cell: only right half colored */
  :global(.rc-cell[data-selection-start]:not([data-selection-end])) {
    background: linear-gradient(to right, transparent 50%, oklch(0.57 0.19 258 / 0.08) 50%);
  }
  /* End cell: only left half colored */
  :global(.rc-cell[data-selection-end]:not([data-selection-start])) {
    background: linear-gradient(to left, transparent 50%, oklch(0.57 0.19 258 / 0.08) 50%);
  }
  /* Same-day start+end: no strip */
  :global(.rc-cell[data-selection-start][data-selection-end]) {
    background: transparent;
  }
  /* Hover preview strip (during selection) */
  :global(.rc-cell[data-highlighted]) {
    background-color: oklch(0.57 0.19 258 / 0.06);
  }

  /* Day button: flatten border-radius for middle/highlighted so strip looks continuous */
  :global(.rc-day[data-range-middle]) {
    border-radius: 0;
    background: transparent;
  }
  :global(.rc-day[data-highlighted]) {
    border-radius: 0;
  }
</style>
