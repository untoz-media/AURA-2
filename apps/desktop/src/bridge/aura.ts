import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppStatus,
  CommandAck,
  CommandRequest,
  CoreError,
  CoreEvent,
  LifecycleEvent,
  PermissionClass,
  PermissionDecision,
  PermissionPolicy,
  RuntimeState,
  ObsConnectRequest,
  ObsConnectionState,
  ObsRuntimeState,
} from "./types";

export const AURA_EVENTS = {
  core: "aura:core-event",
  error: "aura:core-error",
  runtime: "aura:runtime-state",
  lifecycle: "aura:lifecycle-event",
} as const;

export async function getAppStatus(): Promise<AppStatus> {
  return invoke<AppStatus>("get_app_status");
}

export async function getRuntimeState(): Promise<RuntimeState> {
  return invoke<RuntimeState>("get_runtime_state");
}

export async function setRuntimePaused(paused: boolean): Promise<RuntimeState> {
  return invoke<RuntimeState>("set_runtime_paused", { paused });
}

export async function setBackgroundEnabled(
  backgroundEnabled: boolean,
): Promise<RuntimeState> {
  return invoke<RuntimeState>("set_background_enabled", { backgroundEnabled });
}

export async function setAutostartEnabled(
  autostartEnabled: boolean,
): Promise<RuntimeState> {
  return invoke<RuntimeState>("set_autostart_enabled", { autostartEnabled });
}

export async function getPermissionPolicy(): Promise<PermissionPolicy> {
  return invoke<PermissionPolicy>("get_permission_policy");
}

export async function setPermissionDecision(
  permissionClass: PermissionClass,
  decision: PermissionDecision,
): Promise<PermissionPolicy> {
  return invoke<PermissionPolicy>("set_permission_decision", {
    class: permissionClass,
    decision,
  });
}

export async function resetPermissionPolicy(): Promise<PermissionPolicy> {
  return invoke<PermissionPolicy>("reset_permission_policy");
}

export async function resolveConfirmation(
  id: string,
  approved: boolean,
): Promise<CommandAck | null> {
  return invoke<CommandAck | null>("resolve_confirmation", { id, approved });
}

export async function submitAuraCommand(
  request: CommandRequest,
): Promise<CommandAck> {
  return invoke<CommandAck>("process_user_command", { request });
}

export async function openMainWindow(): Promise<void> {
  return invoke<void>("open_main_window");
}

export async function hideOverlay(): Promise<void> {
  return invoke<void>("hide_overlay");
}

export async function listenToOpenSettings(
  handler: () => void,
): Promise<UnlistenFn> {
  return listen("aura:open-settings", () => handler());
}

export async function listenToLifecycle(
  handler: (event: LifecycleEvent) => void,
): Promise<UnlistenFn> {
  return listen<LifecycleEvent>(
    AURA_EVENTS.lifecycle,
    ({ payload }) => handler(payload),
  );
}

export async function listenToAuraCore(
  onEvent: (event: CoreEvent) => void,
  onError: (error: CoreError) => void,
  onRuntime: (state: RuntimeState) => void,
): Promise<UnlistenFn> {
  const unlistenCore = await listen<CoreEvent>(
    AURA_EVENTS.core,
    ({ payload }) => onEvent(payload),
  );

  const unlistenError = await listen<CoreError>(
    AURA_EVENTS.error,
    ({ payload }) => onError(payload),
  );

  const unlistenRuntime = await listen<RuntimeState>(
    AURA_EVENTS.runtime,
    ({ payload }) => onRuntime(payload),
  );

  return () => {
    unlistenCore();
    unlistenError();
    unlistenRuntime();
  };
}


export async function getObsConnectionState(): Promise<ObsConnectionState> {
  return invoke<ObsConnectionState>("get_obs_connection_state");
}

export async function connectObs(
  request: ObsConnectRequest,
): Promise<ObsConnectionState> {
  return invoke<ObsConnectionState>("connect_obs", { request });
}

export async function disconnectObs(): Promise<ObsConnectionState> {
  return invoke<ObsConnectionState>("disconnect_obs");
}


export async function getObsRuntimeState(): Promise<ObsRuntimeState> {
  return invoke<ObsRuntimeState>("get_obs_runtime_state");
}
