import { Copy } from "lucide-react";
import { Button } from "../ui/button";

export function InstructionField({ label, value }: { label: string; value: string }) {
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
          className="h-7 px-2 text-xs"
          onClick={() => void navigator.clipboard.writeText(value)}
        >
          <Copy className="mr-1 h-3.5 w-3.5" />
          Copy
        </Button>
      </div>
    </div>
  );
}
