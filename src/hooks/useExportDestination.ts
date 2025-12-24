import { downloadDir } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import {
  EXPORT_DESTINATION_PREFERENCE,
  getPreference,
  setPreference,
} from "../lib/preferences";

export function useExportDestination(isOpen: boolean) {
  const [destinationDir, setDestinationDir] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    let isActive = true;
    setDestinationDir(null);
    setError(null);

    const loadDestination = async () => {
      try {
        const saved = await getPreference(EXPORT_DESTINATION_PREFERENCE);
        const fallback = saved?.value ?? (await downloadDir());
        if (!isActive) {
          return;
        }
        setDestinationDir(fallback);
      } catch (err) {
        if (!isActive) {
          return;
        }
        let fallbackLoaded = false;
        try {
          const fallback = await downloadDir();
          if (isActive) {
            setDestinationDir(fallback);
            fallbackLoaded = true;
          }
        } catch {
          // Ignore download dir failure and keep destination unset.
        }
        if (!fallbackLoaded) {
          setError("Unable to load saved export destination.");
        }
      }
    };

    void loadDestination();
    return () => {
      isActive = false;
    };
  }, [isOpen]);

  async function selectDestination() {
    setError(null);
    const selection = await open({ directory: true, multiple: false });
    let nextDestination: string | null = null;
    if (typeof selection === "string") {
      nextDestination = selection;
    } else if (Array.isArray(selection) && selection[0]) {
      nextDestination = selection[0];
    }

    if (nextDestination) {
      setDestinationDir(nextDestination);
      try {
        await setPreference(EXPORT_DESTINATION_PREFERENCE, nextDestination);
      } catch (err) {
        console.warn("Failed to save export destination preference.", err);
      }
    }
  }

  async function persistDestination(value: string) {
    try {
      await setPreference(EXPORT_DESTINATION_PREFERENCE, value);
    } catch (err) {
      console.warn("Failed to save export destination preference.", err);
    }
  }

  return {
    destinationDir,
    error,
    selectDestination,
    persistDestination,
  };
}

