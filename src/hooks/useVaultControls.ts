import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { isVaultUnlocked, lockVault, unlockVault } from "../lib/secrets";

const IDLE_TIMEOUT_MS = 5 * 60 * 1000;

export function useVaultControls() {
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

  return {
    vaultUnlocked,
    vaultBusy,
    toggleVault,
  };
}
