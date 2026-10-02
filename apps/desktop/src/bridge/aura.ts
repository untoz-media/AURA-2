import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppStatus,
  CommandAck,
  CommandRequest,
  CoreError,
  CoreEvent,
} from "./types";

export const AURA_EVENTS = {
  core: "aura:core-event",
  error: "aura:core-error",
} as const;

export async function getAppStatus(): Promise<AppStatus> {
  return invoke<AppStatus>("get_app_status");
}

export async function submitAuraCommand(
  request: CommandRequest,
): Promise<CommandAck> {
  return invoke<CommandAck>("process_user_command", { request });
}

export async function listenToAuraCore(
  onEvent: (event: CoreEvent) => void,
  onError: (error: CoreError) => void,
): Promise<UnlistenFn> {
  const unlistenCore = await listen<CoreEvent>(
    AURA_EVENTS.core,
    ({ payload }) => onEvent(payload),
  );

  const unlistenError = await listen<CoreError>(
    AURA_EVENTS.error,
    ({ payload }) => onError(payload),
  );

  return () => {
    unlistenCore();
    unlistenError();
  };
}
