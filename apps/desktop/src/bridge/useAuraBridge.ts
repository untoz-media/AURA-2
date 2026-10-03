import { useCallback, useEffect, useState } from "react";
import {
  connectObs,
  disconnectObs,
  getAppStatus,
  getObsConnectionState,
  getPermissionPolicy,
  getRuntimeState,
  listenToAuraCore,
  listenToLifecycle,
  resetPermissionPolicy,
  resolveConfirmation,
  setAutostartEnabled,
  setBackgroundEnabled,
  setPermissionDecision,
  setRuntimePaused,
  submitAuraCommand,
} from "./aura";
import type {
  AppStatus,
  AuraStatus,
  CoreError,
  CoreEvent,
  LifecycleEvent,
  PermissionClass,
  PermissionDecision,
  PermissionPolicy,
  PendingConfirmation,
  RuntimeState,
  ObsConnectRequest,
  ObsConnectionState,
} from "./types";

const DEFAULT_ACTIVITY =
  "Desktop foundation online. AURA Core and background runtime are ready.";

export function useAuraBridge() {
  const [status, setStatus] = useState<AuraStatus>("Idle");
  const [activity, setActivity] = useState(DEFAULT_ACTIVITY);
  const [appStatus, setAppStatus] = useState<AppStatus | null>(null);
  const [runtimeState, setRuntimeState] = useState<RuntimeState>({
    paused: false,
    backgroundEnabled: true,
    autostartEnabled: false,
  });
  const [obsConnection, setObsConnection] = useState<ObsConnectionState>({
    connected: false,
    host: "127.0.0.1",
    port: 4455,
  });
  const [permissionPolicy, setPermissionPolicyState] = useState<PermissionPolicy>({
    read: "allow",
    act: "allow",
    modify: "ask",
    destructive: "ask",
    sensitive: "ask",
  });
  const [pendingConfirmation, setPendingConfirmation] =
    useState<PendingConfirmation | null>(null);
  const [bridgeError, setBridgeError] = useState<CoreError | null>(null);

  useEffect(() => {
    let cancelled = false;
    let cleanupCore: (() => void) | undefined;
    let cleanupLifecycle: (() => void) | undefined;

    Promise.all([getAppStatus(), getRuntimeState(), getPermissionPolicy(), getObsConnectionState()])
      .then(([app, runtime, permissions, obs]) => {
        if (cancelled) return;
        setAppStatus(app);
        setRuntimeState(runtime);
        setPermissionPolicyState(permissions);
        setObsConnection(obs);

        if (runtime.paused) {
          setActivity("AURA is paused. Resume it from the system tray or settings.");
        } else if (runtime.backgroundEnabled) {
          setActivity("AURA is ready and can remain available in the background.");
        } else {
          setActivity("Background mode is disabled. Closing AURA will quit the app.");
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

        if (
          event.kind === "command.awaiting_confirmation"
          && event.command
        ) {
          setPendingConfirmation({
            id: event.id,
            command: event.command,
            message: event.message,
          });
        } else if (
          [
            "command.confirmed",
            "command.cancelled",
            "command.completed",
            "command.failed",
          ].includes(event.kind)
        ) {
          setPendingConfirmation((current) =>
            current?.id === event.id ? null : current,
          );
        }
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

        if (runtime.paused) {
          setActivity(
            "AURA paused. New commands and future background actions are disabled.",
          );
        } else if (runtime.backgroundEnabled) {
          setActivity("AURA is ready and background mode is enabled.");
        } else {
          setActivity("Background mode is disabled. Closing AURA will quit the app.");
        }
      },
    )
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          cleanupCore = unlisten;
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

    listenToLifecycle((event: LifecycleEvent) => {
      if (cancelled) return;
      setActivity(event.message);
    })
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          cleanupLifecycle = unlisten;
        }
      })
      .catch(() => {
        // Lifecycle messaging is supplementary; the main Core bridge remains authoritative.
      });

    return () => {
      cancelled = true;
      cleanupCore?.();
      cleanupLifecycle?.();
    };
  }, []);

  const submitCommand = useCallback(async (
    text: string,
    source: "desktop" | "overlay" | "voice" = "desktop",
  ) => {
    const trimmed = text.trim();
    if (!trimmed) return null;

    try {
      setBridgeError(null);
      const ack = await submitAuraCommand({
        text: trimmed,
        source,
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

  const setBackgroundMode = useCallback(async (backgroundEnabled: boolean) => {
    const state = await setBackgroundEnabled(backgroundEnabled);
    setRuntimeState(state);
    return state;
  }, []);

  const setAutostart = useCallback(async (autostartEnabled: boolean) => {
    const state = await setAutostartEnabled(autostartEnabled);
    setRuntimeState(state);
    return state;
  }, []);

  const setPermission = useCallback(async (
    permissionClass: PermissionClass,
    decision: PermissionDecision,
  ) => {
    const policy = await setPermissionDecision(permissionClass, decision);
    setPermissionPolicyState(policy);
    return policy;
  }, []);

  const resetPermissions = useCallback(async () => {
    const policy = await resetPermissionPolicy();
    setPermissionPolicyState(policy);
    return policy;
  }, []);

  const connectObsControl = useCallback(async (request: ObsConnectRequest) => {
    try {
      setBridgeError(null);
      const state = await connectObs(request);
      setObsConnection(state);
      setActivity(
        `OBS Studio ${state.obsStudioVersion ?? ""} connected at ${state.host}:${state.port}.`,
      );
      return state;
    } catch (error) {
      const refreshed = await getObsConnectionState().catch(() => ({
        connected: false,
        host: request.host || "127.0.0.1",
        port: request.port || 4455,
        lastError: String(error),
      } satisfies ObsConnectionState));

      setObsConnection(refreshed);
      setBridgeError({
        code: "obs.connection_failed",
        message: String(error),
      });
      setActivity(String(error));
      throw error;
    }
  }, []);

  const disconnectObsControl = useCallback(async () => {
    const state = await disconnectObs();
    setObsConnection(state);
    setBridgeError(null);
    setActivity("OBS Studio disconnected from AURA.");
    return state;
  }, []);

  const approveConfirmation = useCallback(async (id: string) => {
    try {
      setBridgeError(null);
      const ack = await resolveConfirmation(id, true);
      if (ack) {
        setStatus(ack.status);
      }
      return ack;
    } catch (error) {
      const coreError: CoreError = {
        id,
        code: "bridge.confirmation_failed",
        message: String(error),
      };
      setPendingConfirmation(null);
      setBridgeError(coreError);
      setStatus("Idle");
      setActivity(coreError.message);
      return null;
    }
  }, []);

  const cancelConfirmation = useCallback(async (id: string) => {
    try {
      setBridgeError(null);
      await resolveConfirmation(id, false);
    } catch (error) {
      const coreError: CoreError = {
        id,
        code: "bridge.confirmation_failed",
        message: String(error),
      };
      setPendingConfirmation(null);
      setBridgeError(coreError);
      setStatus("Idle");
      setActivity(coreError.message);
    }
  }, []);

  return {
    status,
    activity,
    appStatus,
    runtimeState,
    permissionPolicy,
    obsConnection,
    pendingConfirmation,
    bridgeError,
    submitCommand,
    setPaused,
    setBackgroundMode,
    setAutostart,
    setPermission,
    resetPermissions,
    connectObsControl,
    disconnectObsControl,
    approveConfirmation,
    cancelConfirmation,
  };
}
