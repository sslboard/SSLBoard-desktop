import { Lock, LockOpen, Loader2, Menu } from "lucide-react";
import { Button } from "../ui/button";

type TopbarProps = {
  onMenuToggle: () => void;
  vaultUnlocked: boolean | null;
  vaultBusy: boolean;
  onToggleVault: () => void;
};

export function Topbar({
  onMenuToggle,
  vaultUnlocked,
  vaultBusy,
  onToggleVault,
}: TopbarProps) {
  const vaultLabel =
    vaultUnlocked === null
      ? "Vault status unknown"
      : vaultUnlocked
        ? "Vault unlocked"
        : "Vault locked";

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
        <Button
          variant="outline"
          size="sm"
          onClick={onToggleVault}
          disabled={vaultBusy}
          className="hidden items-center gap-2 sm:inline-flex"
        >
          {vaultBusy ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : vaultUnlocked ? (
            <LockOpen className="h-4 w-4 text-emerald-500" />
          ) : (
            <Lock className="h-4 w-4 text-amber-500" />
          )}
          <span className="font-medium text-foreground">{vaultLabel}</span>
        </Button>
      </div>
    </header>
  );
}
