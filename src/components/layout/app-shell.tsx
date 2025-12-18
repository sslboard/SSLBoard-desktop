import { useEffect, useState } from "react";
import { Sidebar, type NavItem } from "./sidebar";
import { Topbar } from "./topbar";
import { cn } from "../../lib/utils";
import { isVaultUnlocked, lockVault, unlockVault } from "../../lib/secrets";

type AppShellProps = {
  navItems: NavItem[];
  children: React.ReactNode;
};

export function AppShell({ navItems, children }: AppShellProps) {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [vaultUnlocked, setVaultUnlocked] = useState<boolean | null>(null);
  const [vaultBusy, setVaultBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const boot = async () => {
      setVaultBusy(true);
      try {
        const status = await unlockVault();
        if (!cancelled) {
          setVaultUnlocked(status);
        }
      } catch (err) {
        console.error("Failed to unlock vault on startup", err);
        if (!cancelled) {
          try {
            const status = await isVaultUnlocked();
            setVaultUnlocked(status);
          } catch (statusErr) {
            console.error("Failed to check vault status", statusErr);
            setVaultUnlocked(false);
          }
        }
      } finally {
        if (!cancelled) {
          setVaultBusy(false);
        }
      }
    };
    void boot();
    return () => {
      cancelled = true;
    };
  }, []);

  const toggleVault = async () => {
    if (vaultBusy) return;
    setVaultBusy(true);
    try {
      if (vaultUnlocked) {
        const status = await lockVault();
        setVaultUnlocked(status);
      } else {
        const status = await unlockVault();
        setVaultUnlocked(status);
      }
    } catch (err) {
      console.error("Vault toggle failed", err);
    } finally {
      setVaultBusy(false);
    }
  };

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
          <div className="mx-auto max-w-6xl">
            <div className="glass-panel rounded-2xl p-6 sm:p-8">
              {children}
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}
