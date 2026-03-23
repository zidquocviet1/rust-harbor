<script lang="ts">
  import { Calendar, Popover } from "bits-ui";
  import { CalendarDate, getLocalTimeZone } from "@internationalized/date";
  import type { DateValue } from "@internationalized/date";
  import { CalendarIcon, ChevronLeft, ChevronRight } from "lucide-svelte";

  interface Props {
    value: Date | null;
    placeholder?: string;
    onchange: (date: Date | null) => void;
  }

  let { value, placeholder = "Pick a date", onchange }: Props = $props();

  let open = $state(false);

  function toCalendarDate(d: Date | null): DateValue | undefined {
    if (!d) return undefined;
    return new CalendarDate(d.getFullYear(), d.getMonth() + 1, d.getDate());
  }

  const calendarValue = $derived(toCalendarDate(value));

  function formatDate(d: Date | null): string {
    if (!d) return placeholder;
    return d.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  function handleValueChange(v: DateValue | undefined) {
    onchange(v ? v.toDate(getLocalTimeZone()) : null);
    open = false;
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    class="flex items-center gap-2 bg-slate-50 border border-slate-200 rounded-lg px-2.5 py-1.5 text-sm font-medium cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/40 focus:bg-white transition-all hover:bg-white hover:border-slate-300 {!value
      ? 'text-muted-foreground'
      : 'text-foreground border-primary/40'}"
  >
    <CalendarIcon
      size={12}
      class="shrink-0 {value ? 'text-primary' : 'text-muted-foreground'}"
    />
    <span class="whitespace-nowrap">{formatDate(value)}</span>
  </Popover.Trigger>

  <Popover.Content
    class="z-[999999] rounded-xl border border-slate-200 bg-white shadow-2xl p-0 outline-none w-auto"
    sideOffset={6}
    align="start"
  >
    <Calendar.Root
      type="single"
      value={calendarValue}
      onValueChange={handleValueChange}
      weekdayFormat="short"
      class="p-3 select-none"
    >
      {#snippet children({ months, weekdays })}
        {#each months as month}
          <!-- Month navigation header -->
          <div class="relative flex items-center justify-center mb-4">
            <Calendar.PrevButton
              class="absolute left-0 inline-flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-slate-100 transition-colors focus:outline-none disabled:opacity-40"
            >
              <ChevronLeft size={14} />
            </Calendar.PrevButton>

            <Calendar.Heading>
              {#snippet child({ props, headingValue })}
                <div {...props} class="text-sm font-semibold text-foreground px-8">
                  {headingValue}
                </div>
              {/snippet}
            </Calendar.Heading>

            <Calendar.NextButton
              class="absolute right-0 inline-flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-slate-100 transition-colors focus:outline-none disabled:opacity-40"
            >
              <ChevronRight size={14} />
            </Calendar.NextButton>
          </div>

          <!-- Day grid -->
          <Calendar.Grid class="w-full border-collapse">
            <Calendar.GridHead>
              <Calendar.GridRow class="flex mb-1">
                {#each weekdays as weekday}
                  <Calendar.HeadCell
                    class="w-9 h-7 flex items-center justify-center text-[11px] font-medium text-muted-foreground"
                  >
                    {weekday}
                  </Calendar.HeadCell>
                {/each}
              </Calendar.GridRow>
            </Calendar.GridHead>

            <Calendar.GridBody>
              {#each month.weeks as week}
                <Calendar.GridRow class="flex mt-0.5">
                  {#each week as date}
                    <Calendar.Cell {date} month={month.value} class="relative p-0">
                      <Calendar.Day
                        class="inline-flex h-9 w-9 items-center justify-center rounded-lg text-sm transition-colors cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/30 hover:bg-slate-100 data-[selected]:bg-primary data-[selected]:text-primary-foreground data-[selected]:hover:bg-primary/90 data-[today]:font-bold data-[today]:not-data-[selected]:text-primary data-[outside-month]:text-muted-foreground/30 data-[outside-month]:pointer-events-none data-[disabled]:opacity-30 data-[disabled]:pointer-events-none"
                      />
                    </Calendar.Cell>
                  {/each}
                </Calendar.GridRow>
              {/each}
            </Calendar.GridBody>
          </Calendar.Grid>
        {/each}
      {/snippet}
    </Calendar.Root>
  </Popover.Content>
</Popover.Root>
