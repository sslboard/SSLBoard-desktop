import { useState } from "react";
import { Sidebar, type NavItem } from "./sidebar";
import { Topbar } from "./topbar";
import { cn } from "../../lib/utils";
import { useVaultControls } from "../../hooks/useVaultControls";

type AppShellProps = {
  navItems: NavItem[];
  children: React.ReactNode;
};

export function AppShell({ navItems, children }: AppShellProps) {
  const [mobileOpen, setMobileOpen] = useState(false);
  const { vaultUnlocked, vaultBusy, toggleVault } = useVaultControls();

  return (
    <div className="flex min-h-screen bg-background text-foreground">
      <div
        className={cn(
          "fixed inset-y-0 left-0 z-40 w-72 bg-card transition-transform duration-200 lg:relative lg:translate-x-0",
          mobileOpen ? "translate-x-0 shadow-soft" : "-translate-x-full",
        )}
      >
        <Sidebar items={navItems} onNavigate={() => setMobileOpen(false)} />
      </div>
      <div className="flex w-full flex-col lg:ml-0">
        <Topbar
          onMenuToggle={() => setMobileOpen((open) => !open)}
          vaultUnlocked={vaultUnlocked}
          vaultBusy={vaultBusy}
          onToggleVault={toggleVault}
        />
        <main className="flex-1 bg-[radial-gradient(circle_at_top,_rgba(11,113,157,0.08),transparent_45%),radial-gradient(circle_at_80%_20%,_rgba(59,175,213,0.08),transparent_35%)] px-4 pb-8 pt-4 sm:px-6 lg:px-10">
          <div className="mx-4">
            {children}
          </div>
        </main>
      </div>
    </div>
  );
}
