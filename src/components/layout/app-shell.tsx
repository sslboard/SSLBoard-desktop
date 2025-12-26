import { type NavItem } from "./sidebar";
import { Topbar } from "./topbar";
import { useVaultControls } from "../../hooks/useVaultControls";

type AppShellProps = {
  navItems: NavItem[];
  children: React.ReactNode;
};

export function AppShell({ navItems, children }: AppShellProps) {
  const { vaultUnlocked, vaultBusy, toggleVault } = useVaultControls();

  return (
    <div className="flex min-h-screen w-full flex-col bg-background text-foreground">
      <Topbar
        navItems={navItems}
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
  );
}
