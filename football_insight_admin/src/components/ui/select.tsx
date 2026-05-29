import * as React from "react";
import { Check, ChevronDown } from "lucide-react";

import { cn } from "@/lib/utils";

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps {
  id?: string;
  value: string;
  options: SelectOption[];
  onValueChange: (value: string) => void;
  className?: string;
  disabled?: boolean;
}

const Select = React.forwardRef<HTMLButtonElement, SelectProps>(
  ({ id, value, options, onValueChange, className, disabled }, ref) => {
    const [open, setOpen] = React.useState(false);
    const containerRef = React.useRef<HTMLDivElement>(null);
    const selected = options.find((option) => option.value === value);

    React.useEffect(() => {
      if (!open) return;

      function handlePointerDown(event: PointerEvent) {
        if (!containerRef.current?.contains(event.target as Node)) {
          setOpen(false);
        }
      }

      document.addEventListener("pointerdown", handlePointerDown);
      return () => document.removeEventListener("pointerdown", handlePointerDown);
    }, [open]);

    return (
      <div ref={containerRef} className="relative">
        <button
          ref={ref}
          id={id}
          type="button"
          disabled={disabled}
          aria-haspopup="listbox"
          aria-expanded={open}
          className={cn(
            "flex h-9 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-left text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
            className,
          )}
          onClick={() => setOpen((current) => !current)}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              setOpen(false);
            }
          }}
        >
          <span>{selected?.label || value}</span>
          <ChevronDown
            className={cn(
              "h-4 w-4 text-muted-foreground transition-transform",
              open ? "rotate-180" : "",
            )}
          />
        </button>

        {open ? (
          <div
            role="listbox"
            aria-labelledby={id}
            className="absolute left-0 right-0 z-[80] mt-1 overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-lg shadow-zinc-950/10"
          >
            {options.map((option) => {
              const active = option.value === value;
              return (
                <button
                  key={option.value}
                  type="button"
                  role="option"
                  aria-selected={active}
                  className={cn(
                    "flex h-8 w-full items-center justify-between rounded-sm px-2 text-left text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground",
                    active ? "font-medium text-foreground" : "text-muted-foreground",
                  )}
                  onClick={() => {
                    onValueChange(option.value);
                    setOpen(false);
                  }}
                >
                  <span>{option.label}</span>
                  {active ? <Check className="h-4 w-4 text-primary" /> : null}
                </button>
              );
            })}
          </div>
        ) : null}
      </div>
    );
  },
);
Select.displayName = "Select";

export { Select };
