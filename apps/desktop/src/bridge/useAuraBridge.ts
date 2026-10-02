import { useCallback, useEffect, useState } from "react";
import {
  getAppStatus,
  getRuntimeState,
  listenToAuraCore,
  setRuntimePaused,
  submitAuraCommand,
} from "./aura";
import type {
  AppStatus,
  AuraStatus,
  CoreError,
  CoreEvent,
  RuntimeState,
} from "./types";

const DEFAULT_ACTIVITY =
  "Desktop foundation online. AURA Core and system tray are ready.";

export function useAuraBridge() {
  const [status, setStatus] = useState<AuraStatus>("Idle");
  const [activity, setActivity] = useState(DEFAULT_ACTIVITY);
  const [appStatus, setAppStatus] = useState<AppStatus | null>(null);
  const [runtimeState, setRuntimeState] = useState<RuntimeState>({ paused: false });
  const [bridgeError, setBridgeError] = useState<CoreError | null>(null);

  useEffect(() => {
    let cancelled = false;
    let cleanup: (() => void) | undefined;

    Promise.all([getAppStatus(), getRuntimeState()])
      .then(([app, runtime]) => {
        if (cancelled) return;
        setAppStatus(app);
        setRuntimeState(runtime);
        if (runtime.paused) {
          setActivity("AURA is paused. Resume it from the system tray or settings.");
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setBridgeError({
            code: "bridge.status_failed",
            message: String(error),
          });
        }
      });

    listenToAuraCore(
      (event: CoreEvent) => {
        if (cancelled) return;
        setStatus(event.status);
        setActivity(event.message);
        setBridgeError(null);
      },
      (error: CoreError) => {
        if (cancelled) return;
        setStatus("Idle");
        setBridgeError(error);
        setActivity(error.message);
      },
      (runtime: RuntimeState) => {
        if (cancelled) return;
        setRuntimeState(runtime);
        setStatus("Idle");
        setBridgeError(null);
        setActivity(
          runtime.paused
            ? "AURA paused. New commands and future background actions are disabled."
            : "AURA resumed and ready.",
        );
      },
    )
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          cleanup = unlisten;
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setBridgeError({
            code: "bridge.listen_failed",
            message: String(error),
          });
        }
      });

    return () => {
      cancelled = true;
      cleanup?.();
    };
  }, []);

  const submitCommand = useCallback(async (text: string) => {
    const trimmed = text.trim();
    if (!trimmed) return null;

    try {
      setBridgeError(null);
      const ack = await submitAuraCommand({
        text: trimmed,
        source: "desktop",
      });
      setStatus(ack.status);
      return ack;
    } catch (error) {
      const coreError: CoreError = {
        code: "bridge.command_failed",
        message: String(error),
      };
      setBridgeError(coreError);
      setStatus("Idle");
      setActivity(coreError.message);
      return null;
    }
  }, []);

  const setPaused = useCallback(async (paused: boolean) => {
    const state = await setRuntimePaused(paused);
    setRuntimeState(state);
    return state;
  }, []);

  return {
    status,
    activity,
    appStatus,
    runtimeState,
    bridgeError,
    submitCommand,
    setPaused,
  };
}
