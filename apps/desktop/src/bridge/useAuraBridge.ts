import { useCallback, useEffect, useState } from "react";
import {
  getAppStatus,
  listenToAuraCore,
  submitAuraCommand,
} from "./aura";
import type {
  AppStatus,
  AuraStatus,
  CoreError,
  CoreEvent,
} from "./types";

const DEFAULT_ACTIVITY =
  "Desktop foundation online. Core ↔ Desktop bridge is ready.";

export function useAuraBridge() {
  const [status, setStatus] = useState<AuraStatus>("Idle");
  const [activity, setActivity] = useState(DEFAULT_ACTIVITY);
  const [appStatus, setAppStatus] = useState<AppStatus | null>(null);
  const [bridgeError, setBridgeError] = useState<CoreError | null>(null);

  useEffect(() => {
    let cancelled = false;
    let cleanup: (() => void) | undefined;

    getAppStatus()
      .then((value) => {
        if (!cancelled) setAppStatus(value);
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

  return {
    status,
    activity,
    appStatus,
    bridgeError,
    submitCommand,
  };
}
