import { useEffect, useRef, useState } from "react";
import { Sidebar, type NavItem } from "./sidebar";
import { Topbar } from "./topbar";
import { cn } from "../../lib/utils";
import { isVaultUnlocked, lockVault, unlockVault } from "../../lib/secrets";
import { listen } from "@tauri-apps/api/event";

const IDLE_TIMEOUT_MS = 5 * 60 * 1000;

type AppShellProps = {
  navItems: NavItem[];
  children: React.ReactNode;
};

export function AppShell({ navItems, children }: AppShellProps) {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [vaultUnlocked, setVaultUnlocked] = useState<boolean | null>(null);
  const [vaultBusy, setVaultBusy] = useState(false);
  const idleTimer = useRef<number | null>(null);
  const vaultUnlockedRef = useRef(false);
  const lastActivityRef = useRef<number>(Date.now());

  useEffect(() => {
    let cancelled = false;
    const boot = async () => {
      setVaultBusy(true);
      try {
        const status = await isVaultUnlocked();
        if (!cancelled) {
          setVaultUnlocked(status);
        }
      } catch (err) {
        console.error("Failed to check vault status on startup", err);
        if (!cancelled) {
          setVaultUnlocked(false);
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

  useEffect(() => {
    const unlistenPromise = listen<{ unlocked: boolean }>(
      "vault-state-changed",
      (event) => {
        setVaultUnlocked(event.payload.unlocked);
      },
    );
    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const lockIfActive = async () => {
    if (!vaultUnlockedRef.current) return;
    try {
      const status = await lockVault();
      setVaultUnlocked(status);
    } catch (err) {
      console.error("Idle auto-lock failed", err);
    }
  };

  const scheduleIdleCheck = () => {
    if (idleTimer.current !== null) {
      return;
    }
    idleTimer.current = window.setTimeout(function tick() {
      idleTimer.current = null;
      if (!vaultUnlockedRef.current) {
        return;
      }
      const sinceLast = Date.now() - lastActivityRef.current;
      if (sinceLast >= IDLE_TIMEOUT_MS) {
        void lockIfActive();
      } else {
        idleTimer.current = window.setTimeout(tick, IDLE_TIMEOUT_MS - sinceLast);
      }
    }, IDLE_TIMEOUT_MS);
  };

  useEffect(() => {
    vaultUnlockedRef.current = Boolean(vaultUnlocked);
    if (vaultUnlockedRef.current) {
      lastActivityRef.current = Date.now();
      scheduleIdleCheck();
    } else {
      idleTimer.current = null;
    }
  }, [vaultUnlocked]);

  useEffect(() => {
    const handleActivity = () => {
      lastActivityRef.current = Date.now();
      scheduleIdleCheck();
    };
    const handleBlurOrHide = () => {
      void lockIfActive();
    };
    const handleVisibility = () => {
      if (document.hidden) {
        handleBlurOrHide();
      } else {
        handleActivity();
      }
    };

    window.addEventListener("mousemove", handleActivity);
    window.addEventListener("keydown", handleActivity);
    window.addEventListener("click", handleActivity);
    window.addEventListener("focus", handleActivity);
    window.addEventListener("blur", handleBlurOrHide);
    document.addEventListener("visibilitychange", handleVisibility);

    return () => {
      window.removeEventListener("mousemove", handleActivity);
      window.removeEventListener("keydown", handleActivity);
      window.removeEventListener("click", handleActivity);
      window.removeEventListener("focus", handleActivity);
      window.removeEventListener("blur", handleBlurOrHide);
      document.removeEventListener("visibilitychange", handleVisibility);
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
          <div className="mx-4">
            {children}
          </div>
        </main>
      </div>
    </div>
  );
}
