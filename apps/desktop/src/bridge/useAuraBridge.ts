import { useCallback, useEffect, useState } from "react";
import {
  connectObs,
  disconnectObs,
  getAppStatus,
  getObsConnectionState,
  getObsRuntimeState,
  getObsScenes,
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
  setObsProgramScene,
  setObsPreviewScene,
  startObsRecording,
  stopObsRecording,
  pauseObsRecording,
  resumeObsRecording,
  startObsStreaming,
  stopObsStreaming,
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
  ObsRuntimeState,
  ObsSceneList,
  ObsSceneSwitchResult,
  ObsRecordingActionResult,
  ObsStreamingActionResult,
} from "./types";

const DEFAULT_ACTIVITY =
  "Desktop foundation online. AURA Core and background runtime are ready.";

const DEFAULT_OBS_RUNTIME: ObsRuntimeState = {
  available: false,
  streaming: false,
  recording: false,
  recordingPaused: false,
  studioMode: false,
  streamDurationMs: 0,
  streamTimecode: "00:00:00.000",
  refreshedAtMs: 0,
};

const DEFAULT_OBS_SCENES: ObsSceneList = {
  scenes: [],
  refreshedAtMs: 0,
};

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
  const [obsRuntime, setObsRuntime] = useState<ObsRuntimeState>(DEFAULT_OBS_RUNTIME);
  const [obsScenes, setObsScenes] = useState<ObsSceneList>(DEFAULT_OBS_SCENES);
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

  const refreshObsRuntime = useCallback(async () => {
    if (!obsConnection.connected) {
      setObsRuntime(DEFAULT_OBS_RUNTIME);
      return DEFAULT_OBS_RUNTIME;
    }

    const state = await getObsRuntimeState();
    setObsRuntime(state);

    if (!state.available) {
      setObsConnection((current) => ({
        ...current,
        connected: false,
        lastError: state.lastError,
      }));
    }

    return state;
  }, [obsConnection.connected]);


  const refreshObsScenes = useCallback(async () => {
    if (!obsConnection.connected) {
      setObsScenes(DEFAULT_OBS_SCENES);
      return DEFAULT_OBS_SCENES;
    }

    const scenes = await getObsScenes();
    setObsScenes(scenes);
    return scenes;
  }, [obsConnection.connected]);


  const switchObsProgramScene = useCallback(async (
    sceneUuid: string,
  ): Promise<ObsSceneSwitchResult> => {
    try {
      setBridgeError(null);
      const result = await setObsProgramScene({ sceneUuid });
      const [runtime, scenes] = await Promise.all([
        getObsRuntimeState(),
        getObsScenes(),
      ]);
      setObsRuntime(runtime);
      setObsScenes(scenes);
      setActivity(`OBS Program switched to ${result.sceneName}.`);
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.scene_switch_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  const switchObsPreviewScene = useCallback(async (
    sceneUuid: string,
  ): Promise<ObsSceneSwitchResult> => {
    try {
      setBridgeError(null);
      const result = await setObsPreviewScene({ sceneUuid });
      const [runtime, scenes] = await Promise.all([
        getObsRuntimeState(),
        getObsScenes(),
      ]);
      setObsRuntime(runtime);
      setObsScenes(scenes);
      setActivity(`OBS Preview switched to ${result.sceneName}.`);
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.preview_switch_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);


  const controlObsRecording = useCallback(async (
    action: ObsRecordingActionResult["action"],
  ): Promise<ObsRecordingActionResult> => {
    try {
      setBridgeError(null);

      const result = await (
        action === "start"
          ? startObsRecording()
          : action === "stop"
            ? stopObsRecording()
            : action === "pause"
              ? pauseObsRecording()
              : resumeObsRecording()
      );

      const runtime = await getObsRuntimeState();
      setObsRuntime(runtime);

      const message = action === "start"
        ? "OBS recording started."
        : action === "pause"
          ? "OBS recording paused."
          : action === "resume"
            ? "OBS recording resumed."
            : result.outputPath
              ? `OBS recording stopped. Saved to ${result.outputPath}.`
              : "OBS recording stopped.";

      setActivity(message);
      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.recording_control_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);


  const controlObsStreaming = useCallback(async (
    action: ObsStreamingActionResult["action"],
  ): Promise<ObsStreamingActionResult> => {
    try {
      setBridgeError(null);

      const result = await (
        action === "start"
          ? startObsStreaming()
          : stopObsStreaming()
      );

      const runtime = await getObsRuntimeState();
      setObsRuntime(runtime);

      setActivity(
        action === "start"
          ? "OBS stream is live."
          : "OBS stream stopped.",
      );

      return result;
    } catch (error) {
      const message = String(error);
      setBridgeError({
        code: "obs.streaming_control_failed",
        message,
      });
      setActivity(message);
      throw error;
    }
  }, []);

  useEffect(() => {
    if (!obsConnection.connected) {
      setObsRuntime(DEFAULT_OBS_RUNTIME);
      return;
    }

    let cancelled = false;

    const update = async () => {
      try {
        const state = await getObsRuntimeState();
        if (cancelled) return;

        setObsRuntime(state);

        if (!state.available) {
          setObsConnection((current) => ({
            ...current,
            connected: false,
            lastError: state.lastError,
          }));
        }
      } catch (error) {
        if (cancelled) return;

        const message = String(error);
        setObsRuntime({
          ...DEFAULT_OBS_RUNTIME,
          refreshedAtMs: Date.now(),
          lastError: message,
        });
        setObsConnection((current) => ({
          ...current,
          connected: false,
          lastError: message,
        }));
      }
    };

    void update();
    const interval = window.setInterval(() => void update(), 2000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [obsConnection.connected]);

  useEffect(() => {
    if (!obsConnection.connected) {
      setObsScenes(DEFAULT_OBS_SCENES);
      return;
    }

    let cancelled = false;

    const updateScenes = async () => {
      try {
        const scenes = await getObsScenes();
        if (!cancelled) {
          setObsScenes(scenes);
        }
      } catch (error) {
        if (!cancelled) {
          setObsScenes({
            ...DEFAULT_OBS_SCENES,
            refreshedAtMs: Date.now(),
            lastError: String(error),
          });
        }
      }
    };

    void updateScenes();
    const interval = window.setInterval(() => void updateScenes(), 5000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [obsConnection.connected]);

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
      const [runtime, scenes] = await Promise.all([
        getObsRuntimeState(),
        getObsScenes(),
      ]);
      setObsRuntime(runtime);
      setObsScenes(scenes);
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
    setObsRuntime(DEFAULT_OBS_RUNTIME);
    setObsScenes(DEFAULT_OBS_SCENES);
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
    obsRuntime,
    obsScenes,
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
    refreshObsRuntime,
    refreshObsScenes,
    switchObsProgramScene,
    switchObsPreviewScene,
    controlObsRecording,
    controlObsStreaming,
    approveConfirmation,
    cancelConfirmation,
  };
}
