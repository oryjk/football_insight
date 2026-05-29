import * as React from "react";
import { X } from "lucide-react";

import { cn } from "@/lib/utils";

interface DialogProps {
  open: boolean;
  title: string;
  children: React.ReactNode;
  onOpenChange: (open: boolean) => void;
}

export function Dialog({ open, title, children, onOpenChange }: DialogProps) {
  React.useEffect(() => {
    if (!open) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onOpenChange(false);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onOpenChange, open]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950/40 p-4">
      <div
        className="absolute inset-0"
        aria-hidden="true"
        onClick={() => onOpenChange(false)}
      />
      <section
        className={cn(
          "relative z-10 w-full max-w-xl rounded-lg border border-border bg-card p-5 text-card-foreground shadow-2xl",
        )}
        role="dialog"
        aria-modal="true"
        aria-label={title}
      >
        <div className="mb-5 flex items-center justify-between gap-3">
          <h2 className="text-base font-semibold">{title}</h2>
          <button
            type="button"
            className="rounded-md p-1 text-muted-foreground hover:bg-secondary hover:text-foreground"
            onClick={() => onOpenChange(false)}
            aria-label="关闭"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
        {children}
      </section>
    </div>
  );
}
