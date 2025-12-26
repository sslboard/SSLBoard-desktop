import { useEffect, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "../ui/button";
import { cn } from "../../lib/utils";

export function InstructionField({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);
  const resetTimer = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (resetTimer.current !== null) {
        window.clearTimeout(resetTimer.current);
      }
    };
  }, []);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      if (resetTimer.current !== null) {
        window.clearTimeout(resetTimer.current);
      }
      resetTimer.current = window.setTimeout(() => {
        setCopied(false);
      }, 1200);
    } catch {
      // Ignore clipboard errors; some environments may block access.
    }
  };

  return (
    <div className="space-y-1">
      <div className="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
        {label}
      </div>
      <div className="flex items-center justify-between gap-2 rounded-md border bg-muted/60 px-3 py-2 font-mono text-xs">
        <span className="truncate">{value}</span>
        <Button
          variant="ghost"
          size="sm"
          className={cn(
            "h-7 px-2 text-xs transition-colors",
            copied && "animate-pulse bg-primary/10 text-primary hover:bg-primary/10",
          )}
          onClick={() => void handleCopy()}
        >
          {copied ? (
            <Check className="mr-1 h-3.5 w-3.5" />
          ) : (
            <Copy className="mr-1 h-3.5 w-3.5" />
          )}
          {copied ? "Copied" : "Copy"}
        </Button>
      </div>
    </div>
  );
}
