import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppStatus,
  CommandAck,
  CommandRequest,
  CoreError,
  CoreEvent,
  LifecycleEvent,
  RuntimeState,
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
