"use client";
import { useState, useRef, useEffect } from "react";
import { DayPicker, type Matcher } from "react-day-picker";
import { ptBR } from "react-day-picker/locale";
import { MdCalendarToday, MdChevronLeft, MdChevronRight } from "react-icons/md";

interface Props {
  value: string;
  onChange: (v: string) => void;
  label?: string;
  required?: boolean;
  min?: string;
  max?: string;
}

function isoToDate(iso: string): Date | undefined {
  if (!iso || !/^\d{4}-\d{2}-\d{2}$/.test(iso)) return undefined;
  const [y, m, d] = iso.split("-").map(Number);
  return new Date(y, m - 1, d);
}

function dateToIso(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function isoToDisplay(iso: string): string {
  if (!iso || !/^\d{4}-\d{2}-\d{2}$/.test(iso)) return "";
  return `${iso.slice(8, 10)}/${iso.slice(5, 7)}/${iso.slice(0, 4)}`;
}

export default function InputData({ value, onChange, label, required, min, max }: Props) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const selected = isoToDate(value);

  const disabledMatchers: Matcher[] = [];
  if (min) { const d = isoToDate(min); if (d) disabledMatchers.push({ before: d }); }
  if (max) { const d = isoToDate(max); if (d) disabledMatchers.push({ after: d }); }

  useEffect(() => {
    function onOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener("mousedown", onOutside);
    return () => document.removeEventListener("mousedown", onOutside);
  }, []);

  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <div className="relative" ref={ref}>
        <input
          className="input w-full pr-10 cursor-pointer"
          type="text"
          readOnly
          value={isoToDisplay(value)}
          placeholder="dd/mm/aaaa"
          required={required}
          onClick={() => setOpen((v) => !v)}
        />
        <span className="absolute right-3 top-1/2 -translate-y-1/2 text-base-content/50 pointer-events-none">
          <MdCalendarToday />
        </span>
        {open && (
          <div className="absolute z-200 mt-1 bg-base-200 rounded-box shadow-xl border border-base-300 p-3 min-w-65">
            <DayPicker
              mode="single"
              locale={ptBR}
              selected={selected}
              defaultMonth={selected ?? new Date()}
              onSelect={(date) => {
                onChange(date ? dateToIso(date) : "");
                setOpen(false);
              }}
              disabled={disabledMatchers.length > 0 ? disabledMatchers : undefined}
              components={{
                Chevron: ({ orientation }) =>
                  orientation === "left"
                    ? <MdChevronLeft size={18} />
                    : <MdChevronRight size={18} />,
              }}
              classNames={{
                month_caption: "relative flex justify-center items-center py-1 mb-2",
                caption_label: "text-sm font-semibold",
                nav: "absolute inset-x-0 top-0 flex justify-between",
                button_previous: "btn btn-ghost btn-xs btn-circle",
                button_next: "btn btn-ghost btn-xs btn-circle",
                month_grid: "w-full",
                weekdays: "flex",
                weekday: "w-9 text-center text-xs font-medium text-base-content/50 pb-1",
                week: "flex",
                day: "p-0",
                day_button: "w-9 h-9 rounded-full flex items-center justify-center text-sm cursor-pointer hover:bg-base-300 transition-colors",
                selected: "!bg-primary !text-primary-content hover:!bg-primary/90",
                today: "!font-bold !border !border-primary",
                outside: "!opacity-30",
                disabled: "!opacity-20 !pointer-events-none",
              }}
            />
          </div>
        )}
      </div>
    </fieldset>
  );
}
