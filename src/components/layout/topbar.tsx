import { Menu, ShieldCheck, Sparkles } from "lucide-react";
import { Button } from "../ui/button";

type TopbarProps = {
  onMenuToggle: () => void;
};

export function Topbar({ onMenuToggle }: TopbarProps) {
  return (
    <header className="sticky top-0 z-30 flex h-14 w-full items-center justify-between border-b bg-card/95 px-4 backdrop-blur">
      <div className="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          className="lg:hidden"
          onClick={onMenuToggle}
          aria-label="Toggle navigation"
        >
          <Menu className="h-5 w-5" />
        </Button>
      </div>
      <div className="flex items-center gap-3 text-sm text-muted-foreground">
        <div className="hidden items-center gap-2 rounded-full border px-3 py-1 sm:flex">
          <Sparkles className="h-4 w-4 text-primary" />
          <span className="font-semibold text-foreground">Preview Shell</span>
        </div>
        <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
          <ShieldCheck className="h-5 w-5" />
        </div>
      </div>
    </header>
  );
}
