import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { lockVault } from "../lib/secrets";

const IDLE_TIMEOUT_MS = 5 * 60 * 1000; // 5 minutes
const BLUR_LOCK_DELAY_MS = 10_000; // 10 seconds

export function useVaultControls() {
  const [vaultUnlocked, setVaultUnlocked] = useState<boolean>(false);
  const idleTimer = useRef<number | null>(null);
  const blurTimer = useRef<number | null>(null);
  const vaultUnlockedRef = useRef(false);
  const lastActivityRef = useRef<number>(Date.now());

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
      await lockVault();
      setVaultUnlocked(false);
    } catch (err) {
      console.error("Idle auto-lock failed", err);
    }
  };

  const scheduleIdleCheck = () => {
    if (!vaultUnlockedRef.current) {
      return;
    }
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
      if (idleTimer.current !== null) {
        window.clearTimeout(idleTimer.current);
        idleTimer.current = null;
      }
      if (blurTimer.current !== null) {
        window.clearTimeout(blurTimer.current);
        blurTimer.current = null;
      }
    }
  }, [vaultUnlocked]);

  useEffect(() => {
    const handleActivity = () => {
      lastActivityRef.current = Date.now();
      if (blurTimer.current !== null) {
        window.clearTimeout(blurTimer.current);
        blurTimer.current = null;
      }
      scheduleIdleCheck();
    };
    const handleBlurOrHide = () => {
      if (blurTimer.current !== null) {
        return;
      }
      blurTimer.current = window.setTimeout(() => {
        blurTimer.current = null;
        void lockIfActive();
      }, BLUR_LOCK_DELAY_MS);
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
    window.addEventListener("wheel", handleActivity, { passive: true });
    window.addEventListener("touchstart", handleActivity, { passive: true });
    window.addEventListener("pointerdown", handleActivity);
    window.addEventListener("focus", handleActivity);
    window.addEventListener("blur", handleBlurOrHide);
    document.addEventListener("visibilitychange", handleVisibility);

    return () => {
      window.removeEventListener("mousemove", handleActivity);
      window.removeEventListener("keydown", handleActivity);
      window.removeEventListener("click", handleActivity);
      window.removeEventListener("wheel", handleActivity);
      window.removeEventListener("touchstart", handleActivity);
      window.removeEventListener("pointerdown", handleActivity);
      window.removeEventListener("focus", handleActivity);
      window.removeEventListener("blur", handleBlurOrHide);
      document.removeEventListener("visibilitychange", handleVisibility);
      if (idleTimer.current !== null) {
        window.clearTimeout(idleTimer.current);
        idleTimer.current = null;
      }
      if (blurTimer.current !== null) {
        window.clearTimeout(blurTimer.current);
        blurTimer.current = null;
      }
    };
  }, []);

  return {
    vaultUnlocked,
  };
}
