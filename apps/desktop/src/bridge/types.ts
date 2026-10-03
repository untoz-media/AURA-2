export type AuraStatus = "Idle" | "Listening" | "Thinking" | "Working" | "Waiting";

export type CoreEventKind =
  | "command.accepted"
  | "command.processing"
  | "command.completed"
  | "command.failed"
  | "command.awaiting_confirmation"
  | "command.confirmed"
  | "command.cancelled"
  | "command.unhandled";

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
  approvalId?: string;
};

export type PendingConfirmation = {
  id: string;
  command: string;
  message: string;
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
  | "startup.background"
  | "permissions.updated"
  | "permissions.reset";

export type LifecycleEvent = {
  kind: LifecycleEventKind;
  message: string;
  timestampMs: number;
};

export type PermissionClass =
  | "read"
  | "act"
  | "modify"
  | "destructive"
  | "sensitive";

export type PermissionDecision = "allow" | "ask" | "never";

export type PermissionPolicy = {
  read: PermissionDecision;
  act: PermissionDecision;
  modify: PermissionDecision;
  destructive: PermissionDecision;
  sensitive: PermissionDecision;
};


export type ObsConnectRequest = {
  host?: string;
  port?: number;
  password?: string;
};

export type ObsConnectionState = {
  connected: boolean;
  host: string;
  port: number;
  obsStudioVersion?: string;
  obsWebsocketVersion?: string;
  rpcVersion?: number;
  connectedAtMs?: number;
  lastError?: string;
};


export type ObsRuntimeState = {
  available: boolean;
  streaming: boolean;
  recording: boolean;
  recordingPaused: boolean;
  studioMode: boolean;
  currentProgramScene?: string;
  currentPreviewScene?: string;
  refreshedAtMs: number;
  lastError?: string;
};
