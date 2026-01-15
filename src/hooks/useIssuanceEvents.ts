import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

export type IssuanceStep =
  | "starting"
  | "challenges-received"
  | "dns-verification"
  | "finalization"
  | "complete";

export type IssuanceStatus = "started" | "complete";

export interface IssuanceEventPayload {
  request_id: string;
  step: IssuanceStep;
  status: IssuanceStatus;
  error: string | null;
}

export interface IssuanceEventState {
  requestId: string | null;
  step: "idle" | "starting" | "challenges-received" | "dns-verification" | "dns-complete" | "finalization" | "complete";
  error: string | null;
}

export function useIssuanceEvents(
  onEvent: (state: IssuanceEventState) => void,
  isActive: boolean = true,
) {
  const stateRef = useRef<IssuanceEventState>({
    requestId: null,
    step: "idle",
    error: null,
  });
  const onEventRef = useRef(onEvent);
  
  // Keep onEvent ref in sync
  useEffect(() => {
    onEventRef.current = onEvent;
  }, [onEvent]);

  useEffect(() => {
    if (!isActive) {
      // Reset state when not active
      stateRef.current = {
        requestId: null,
        step: "idle",
        error: null,
      };
      onEvent(stateRef.current);
      return;
    }

    // Listen to all issuance-progress events (only one active issuance at a time)
    const unlistenPromise = listen<IssuanceEventPayload>("issuance-progress", (event) => {
      console.log("[IssuanceEvents] issuance-progress:", event.payload);
      
      // Process all events since only one issuance is active at a time
      const { step, status, error } = event.payload;
      let mappedStep: IssuanceEventState["step"];

      if (step === "dns-verification" && status === "complete" && !error) {
        mappedStep = "dns-complete";
      } else if (step === "dns-verification" && status === "complete" && error) {
        mappedStep = "dns-verification";
      } else if (step === "finalization" && status === "complete" && !error) {
        mappedStep = "complete";
      } else if (step === "finalization" && status === "complete" && error) {
        mappedStep = "finalization";
      } else {
        mappedStep = step;
      }

      stateRef.current = {
        requestId: event.payload.request_id,
        step: mappedStep,
        error: error,
      };
      onEventRef.current(stateRef.current);
    });

    return () => {
      // Cleanup: unlisten from event
      void unlistenPromise.then((unlisten) => {
        unlisten();
      });
    };
  }, [onEvent, isActive]);
}
