import { useState } from "react";
import type {
  DirectorPreset,
  DirectorPresetAction,
  DirectorPresetRunResult,
  DirectorRecordingAction,
  DirectorStreamingAction,
  ObsAudioInputList,
  ObsSceneList,
  ObsSourceItemList,
  SaveDirectorPresetRequest,
} from "./bridge/types";
import "./director-presets.css";

type Props = {
  connected: boolean;
  scenes: ObsSceneList;
  sources: ObsSourceItemList;
  audio: ObsAudioInputList;
  presets: DirectorPreset[];
  lastRun: DirectorPresetRunResult | null;
  onRefresh: () => Promise<DirectorPreset[]>;
  onSave: (request: SaveDirectorPresetRequest) => Promise<DirectorPreset>;
  onDelete: (presetId: string) => Promise<void>;
  onRun: (presetId: string) => Promise<DirectorPresetRunResult>;
};

type ActionType = DirectorPresetAction["type"];

const ACTION_TYPES: Array<{ value: ActionType; label: string }> = [
  { value: "programScene", label: "Program Scene" },
  { value: "previewScene", label: "Preview Scene" },
  { value: "sourceVisibility", label: "Source Visibility" },
  { value: "audioMute", label: "Audio Mute" },
  { value: "audioVolume", label: "Audio Volume" },
  { value: "recording", label: "Recording" },
  { value: "streaming", label: "Streaming" },
  { value: "wait", label: "Wait" },
];

function isSensitivePreset(preset: DirectorPreset) {
  return preset.actions.some(
    (action) => action.type === "streaming" && action.action === "start",
  );
}

function describeAction(action: DirectorPresetAction) {
  switch (action.type) {
    case "programScene":
      return `Program → ${action.sceneName}`;
    case "previewScene":
      return `Preview → ${action.sceneName}`;
    case "sourceVisibility":
      return `${action.enabled ? "Show" : "Hide"} ${action.sourceName}`;
    case "audioMute":
      return `${action.muted ? "Mute" : "Unmute"} ${action.inputName}`;
    case "audioVolume":
      return `${action.inputName} → ${action.percent}%`;
    case "recording":
      return `Recording → ${action.action}`;
    case "streaming":
      return `Streaming → ${action.action}`;
    case "wait":
      return `Wait ${action.milliseconds} ms`;
  }
}

function splitAliases(value: string) {
  const seen = new Set<string>();

  return value
    .split(",")
    .map((alias) => alias.trim())
    .filter(Boolean)
    .filter((alias) => {
      const key = alias.toLowerCase();
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
}

export default function DirectorPresets({
  connected,
  scenes,
  sources,
  audio,
  presets,
  lastRun,
  onRefresh,
  onSave,
  onDelete,
  onRun,
}: Props) {
  const [editing, setEditing] = useState(false);
  const [editingId, setEditingId] = useState<string | undefined>();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [aliases, setAliases] = useState("");
  const [actions, setActions] = useState<DirectorPresetAction[]>([]);

  const [actionType, setActionType] = useState<ActionType>("programScene");
  const [targetName, setTargetName] = useState("");
  const [enabled, setEnabled] = useState(true);
  const [muted, setMuted] = useState(true);
  const [percent, setPercent] = useState(70);
  const [recordingAction, setRecordingAction] =
    useState<DirectorRecordingAction>("start");
  const [streamingAction, setStreamingAction] =
    useState<DirectorStreamingAction>("start");
  const [waitMs, setWaitMs] = useState(1000);

  const [saving, setSaving] = useState(false);
  const [runningId, setRunningId] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [localError, setLocalError] = useState<string | null>(null);

  function resetEditor() {
    setEditing(false);
    setEditingId(undefined);
    setName("");
    setDescription("");
    setAliases("");
    setActions([]);
    setActionType("programScene");
    setTargetName("");
    setEnabled(true);
    setMuted(true);
    setPercent(70);
    setRecordingAction("start");
    setStreamingAction("start");
    setWaitMs(1000);
    setLocalError(null);
  }

  function newPreset() {
    resetEditor();
    setEditing(true);
  }

  function editPreset(preset: DirectorPreset) {
    setEditing(true);
    setEditingId(preset.id);
    setName(preset.name);
    setDescription(preset.description);
    setAliases(preset.aliases.join(", "));
    setActions(preset.actions.map((action) => ({ ...action })));
    setLocalError(null);
  }

  function addAction() {
    setLocalError(null);

    const target = targetName.trim();
    let action: DirectorPresetAction | null = null;

    switch (actionType) {
      case "programScene":
        if (!target) return setLocalError("Choose or enter a Program scene.");
        action = { type: "programScene", sceneName: target };
        break;
      case "previewScene":
        if (!target) return setLocalError("Choose or enter a Preview scene.");
        action = { type: "previewScene", sceneName: target };
        break;
      case "sourceVisibility":
        if (!target) return setLocalError("Choose or enter an OBS source.");
        action = { type: "sourceVisibility", sourceName: target, enabled };
        break;
      case "audioMute":
        if (!target) return setLocalError("Choose or enter an OBS audio input.");
        action = { type: "audioMute", inputName: target, muted };
        break;
      case "audioVolume":
        if (!target) return setLocalError("Choose or enter an OBS audio input.");
        if (!Number.isInteger(percent) || percent < 0 || percent > 100) {
          return setLocalError("Audio volume must be between 0 and 100%.");
        }
        action = { type: "audioVolume", inputName: target, percent };
        break;
      case "recording":
        action = { type: "recording", action: recordingAction };
        break;
      case "streaming":
        action = { type: "streaming", action: streamingAction };
        break;
      case "wait":
        if (!Number.isInteger(waitMs) || waitMs < 0 || waitMs > 30_000) {
          return setLocalError("Wait must be between 0 and 30,000 ms.");
        }
        action = { type: "wait", milliseconds: waitMs };
        break;
    }

    setActions((current) => [...current, action]);
    setTargetName("");
  }

  function moveAction(index: number, direction: -1 | 1) {
    const destination = index + direction;
    if (destination < 0 || destination >= actions.length) return;

    setActions((current) => {
      const next = [...current];
      [next[index], next[destination]] = [next[destination], next[index]];
      return next;
    });
  }

  async function savePreset() {
    setLocalError(null);

    if (!name.trim()) {
      setLocalError("Preset name is required.");
      return;
    }

    if (actions.length === 0) {
      setLocalError("Add at least one Director Mode step.");
      return;
    }

    setSaving(true);
    try {
      await onSave({
        id: editingId,
        name: name.trim(),
        description: description.trim(),
        aliases: splitAliases(aliases),
        actions,
      });
      resetEditor();
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setSaving(false);
    }
  }

  async function deletePreset(preset: DirectorPreset) {
    if (!window.confirm(`Delete Director Mode preset “${preset.name}”?`)) {
      return;
    }

    setDeletingId(preset.id);
    setLocalError(null);
    try {
      await onDelete(preset.id);
      if (editingId === preset.id) resetEditor();
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setDeletingId(null);
    }
  }

  async function runPreset(preset: DirectorPreset) {
    setRunningId(preset.id);
    setLocalError(null);
    try {
      const result = await onRun(preset.id);
      if (!result.success) {
        setLocalError(
          result.error ??
            `Preset stopped at step ${(result.failedStep ?? 0) + 1}.`,
        );
      }
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setRunningId(null);
    }
  }

  return (
    <div className="director-panel">
      <datalist id="director-scene-options">
        {scenes.scenes.map((scene) => (
          <option value={scene.name} key={scene.uuid} />
        ))}
      </datalist>
      <datalist id="director-source-options">
        {sources.items.map((source) => (
          <option
            value={source.sourceName}
            key={`${source.sceneName}:${source.itemId}`}
          />
        ))}
      </datalist>
      <datalist id="director-audio-options">
        {audio.inputs.map((input) => (
          <option value={input.inputName} key={input.inputUuid} />
        ))}
      </datalist>

      <div className="director-heading">
        <div>
          <span className="director-kicker">DIRECTOR MODE</span>
          <strong>Production Presets</strong>
          <p>
            Chain OBS actions into reusable, local routines. Presets run in order
            and stop immediately if a step fails.
          </p>
        </div>
        <div className="director-heading-actions">
          <button
            type="button"
            className="settings-action-button"
            onClick={() => void onRefresh()}
          >
            Refresh
          </button>
          <button
            type="button"
            className="settings-action-button director-primary"
            onClick={newPreset}
          >
            New preset
          </button>
        </div>
      </div>

      {presets.length > 0 ? (
        <div className="director-preset-grid">
          {presets.map((preset) => {
            const sensitive = isSensitivePreset(preset);
            return (
              <article className="director-preset-card" key={preset.id}>
                <div className="director-preset-card-head">
                  <div>
                    <strong>{preset.name}</strong>
                    <span>
                      {preset.actions.length} step
                      {preset.actions.length === 1 ? "" : "s"}
                    </span>
                  </div>
                  <div className="director-tags">
                    {sensitive && (
                      <span className="director-tag sensitive">Can go live</span>
                    )}
                    <span className="director-tag">
                      {sensitive ? "Sensitive" : "Act"}
                    </span>
                  </div>
                </div>

                {preset.description && <p>{preset.description}</p>}

                {preset.aliases.length > 0 && (
                  <div className="director-aliases">
                    {preset.aliases.map((alias) => (
                      <span key={alias}>{alias}</span>
                    ))}
                  </div>
                )}

                <div className="director-preview-steps">
                  {preset.actions.slice(0, 4).map((action, index) => (
                    <span key={`${preset.id}:${index}`}>
                      {index + 1}. {describeAction(action)}
                    </span>
                  ))}
                  {preset.actions.length > 4 && (
                    <span>+{preset.actions.length - 4} more</span>
                  )}
                </div>

                <div className="director-card-actions">
                  <button
                    type="button"
                    className="settings-action-button director-primary"
                    disabled={!connected || runningId !== null}
                    title={
                      connected
                        ? undefined
                        : "Connect AURA to OBS before running a preset."
                    }
                    onClick={() => void runPreset(preset)}
                  >
                    {runningId === preset.id ? "Running…" : "Run"}
                  </button>
                  <button
                    type="button"
                    className="settings-action-button"
                    disabled={runningId !== null || saving}
                    onClick={() => editPreset(preset)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    className="settings-action-button director-danger"
                    disabled={deletingId !== null || runningId !== null}
                    onClick={() => void deletePreset(preset)}
                  >
                    {deletingId === preset.id ? "Deleting…" : "Delete"}
                  </button>
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <div className="director-empty">
          No Director Mode presets yet. Create one to turn a production routine
          into a single command.
        </div>
      )}

      {editing && (
        <div className="director-editor">
          <div className="director-editor-heading">
            <div>
              <strong>{editingId ? "Edit preset" : "New preset"}</strong>
              <span>Build the exact sequence AURA should execute.</span>
            </div>
            <button
              type="button"
              className="settings-action-button"
              onClick={resetEditor}
            >
              Cancel
            </button>
          </div>

          <div className="director-fields">
            <label className="settings-field">
              <span>Name</span>
              <input
                value={name}
                maxLength={64}
                placeholder="Prepare Match"
                onChange={(event) => setName(event.target.value)}
              />
            </label>
            <label className="settings-field">
              <span>Aliases</span>
              <input
                value={aliases}
                placeholder="prepare the match, ready match"
                onChange={(event) => setAliases(event.target.value)}
              />
            </label>
            <label className="settings-field director-description-field">
              <span>Description</span>
              <input
                value={description}
                maxLength={240}
                placeholder="Prepare the full match production."
                onChange={(event) => setDescription(event.target.value)}
              />
            </label>
          </div>

          <div className="director-action-builder">
            <label className="settings-field compact">
              <span>Action</span>
              <select
                value={actionType}
                onChange={(event) =>
                  setActionType(event.target.value as ActionType)
                }
              >
                {ACTION_TYPES.map((item) => (
                  <option value={item.value} key={item.value}>
                    {item.label}
                  </option>
                ))}
              </select>
            </label>

            {(actionType === "programScene" ||
              actionType === "previewScene") && (
              <label className="settings-field director-builder-target">
                <span>Scene</span>
                <input
                  list="director-scene-options"
                  value={targetName}
                  placeholder="Intro"
                  onChange={(event) => setTargetName(event.target.value)}
                />
              </label>
            )}

            {actionType === "sourceVisibility" && (
              <>
                <label className="settings-field director-builder-target">
                  <span>Source</span>
                  <input
                    list="director-source-options"
                    value={targetName}
                    placeholder="Scoreboard"
                    onChange={(event) => setTargetName(event.target.value)}
                  />
                </label>
                <label className="settings-field compact">
                  <span>State</span>
                  <select
                    value={enabled ? "show" : "hide"}
                    onChange={(event) => setEnabled(event.target.value === "show")}
                  >
                    <option value="show">Show</option>
                    <option value="hide">Hide</option>
                  </select>
                </label>
              </>
            )}

            {actionType === "audioMute" && (
              <>
                <label className="settings-field director-builder-target">
                  <span>Audio input</span>
                  <input
                    list="director-audio-options"
                    value={targetName}
                    placeholder="Mic/Aux"
                    onChange={(event) => setTargetName(event.target.value)}
                  />
                </label>
                <label className="settings-field compact">
                  <span>State</span>
                  <select
                    value={muted ? "mute" : "unmute"}
                    onChange={(event) => setMuted(event.target.value === "mute")}
                  >
                    <option value="mute">Mute</option>
                    <option value="unmute">Unmute</option>
                  </select>
                </label>
              </>
            )}

            {actionType === "audioVolume" && (
              <>
                <label className="settings-field director-builder-target">
                  <span>Audio input</span>
                  <input
                    list="director-audio-options"
                    value={targetName}
                    placeholder="Mic/Aux"
                    onChange={(event) => setTargetName(event.target.value)}
                  />
                </label>
                <label className="settings-field compact">
                  <span>Volume %</span>
                  <input
                    type="number"
                    min="0"
                    max="100"
                    value={percent}
                    onChange={(event) => setPercent(Number(event.target.value))}
                  />
                </label>
              </>
            )}

            {actionType === "recording" && (
              <label className="settings-field director-builder-target">
                <span>Recording</span>
                <select
                  value={recordingAction}
                  onChange={(event) =>
                    setRecordingAction(
                      event.target.value as DirectorRecordingAction,
                    )
                  }
                >
                  <option value="start">Start</option>
                  <option value="stop">Stop</option>
                  <option value="pause">Pause</option>
                  <option value="resume">Resume</option>
                </select>
              </label>
            )}

            {actionType === "streaming" && (
              <label className="settings-field director-builder-target">
                <span>Streaming</span>
                <select
                  value={streamingAction}
                  onChange={(event) =>
                    setStreamingAction(
                      event.target.value as DirectorStreamingAction,
                    )
                  }
                >
                  <option value="start">Go Live</option>
                  <option value="stop">Stop Stream</option>
                </select>
              </label>
            )}

            {actionType === "wait" && (
              <label className="settings-field director-builder-target">
                <span>Wait (ms)</span>
                <input
                  type="number"
                  min="0"
                  max="30000"
                  step="100"
                  value={waitMs}
                  onChange={(event) => setWaitMs(Number(event.target.value))}
                />
              </label>
            )}

            <button
              type="button"
              className="settings-action-button director-add-step"
              onClick={addAction}
            >
              Add step
            </button>
          </div>

          <p className="director-builder-note">
            Source visibility targets the Program scene at that point in the
            sequence. Put a Program Scene step first when the source belongs to
            a different scene.
          </p>

          {actions.length > 0 ? (
            <div className="director-action-list">
              {actions.map((action, index) => (
                <div className="director-action-row" key={`${index}:${describeAction(action)}`}>
                  <span className="director-step-number">
                    {String(index + 1).padStart(2, "0")}
                  </span>
                  <strong>{describeAction(action)}</strong>
                  <div className="director-action-row-buttons">
                    <button
                      type="button"
                      aria-label="Move step up"
                      disabled={index === 0}
                      onClick={() => moveAction(index, -1)}
                    >
                      ↑
                    </button>
                    <button
                      type="button"
                      aria-label="Move step down"
                      disabled={index === actions.length - 1}
                      onClick={() => moveAction(index, 1)}
                    >
                      ↓
                    </button>
                    <button
                      type="button"
                      aria-label="Remove step"
                      onClick={() =>
                        setActions((current) =>
                          current.filter((_, actionIndex) => actionIndex !== index),
                        )
                      }
                    >
                      ×
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="director-empty compact">
              Add the first action to this preset.
            </div>
          )}

          <div className="director-editor-footer">
            <span>
              {actions.length}/32 steps
              {actions.some(
                (action) =>
                  action.type === "streaming" && action.action === "start",
              )
                ? " · Sensitive: can start an external stream"
                : ""}
            </span>
            <button
              type="button"
              className="settings-action-button director-primary"
              disabled={saving}
              onClick={() => void savePreset()}
            >
              {saving ? "Saving…" : editingId ? "Save changes" : "Create preset"}
            </button>
          </div>
        </div>
      )}

      {lastRun && (
        <div className="director-last-run">
          <div className="director-last-run-head">
            <div>
              <strong>Last run · {lastRun.presetName}</strong>
              <span>
                {lastRun.completedSteps}/{lastRun.totalSteps} steps completed
              </span>
            </div>
            <span
              className={`director-run-status ${lastRun.success ? "success" : "failed"}`}
            >
              {lastRun.success ? "Completed" : "Stopped"}
            </span>
          </div>
          <div className="director-run-steps">
            {lastRun.steps.map((step) => (
              <div key={`${lastRun.startedAtMs}:${step.index}`}>
                <span>{String(step.index + 1).padStart(2, "0")}</span>
                <strong>{step.label}</strong>
                <em className={step.status}>{step.status}</em>
                <p>{step.message}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {localError && <p className="director-error">{localError}</p>}
    </div>
  );
}
