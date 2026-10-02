export type AuraStatus = "Idle" | "Listening" | "Thinking" | "Working" | "Waiting";

export type CoreEventKind =
  | "command.accepted"
  | "command.processing"
  | "command.completed"
  | "command.failed";

export type CoreEvent = {
  id: string;
  kind: CoreEventKind;
  status: AuraStatus;
  message: string;
  command?: string;
  timestampMs: number;
};

export type CommandRequest = {
  text: string;
  source: "desktop" | "overlay" | "voice";
};

export type CommandAck = {
  id: string;
  accepted: boolean;
  status: AuraStatus;
};

export type AppStatus = {
  name: string;
  version: string;
  stage: string;
  localFirst: boolean;
};

export type CoreError = {
  id?: string;
  code: string;
  message: string;
};

export type RuntimeState = {
  paused: boolean;
  backgroundEnabled: boolean;
  autostartEnabled: boolean;
};

export type LifecycleEventKind =
  | "foreground.entered"
  | "background.entered"
  | "background.enabled"
  | "background.disabled"
  | "autostart.enabled"
  | "autostart.disabled"
  | "startup.background";

export type LifecycleEvent = {
  kind: LifecycleEventKind;
  message: string;
  timestampMs: number;
};
