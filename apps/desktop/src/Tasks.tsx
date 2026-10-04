import { FormEvent, useMemo, useState } from "react";
import type {
  RoutineRunResult,
  RoutineStep,
  SaveRoutineRequest,
  UserRoutine,
} from "./bridge/types";
import "./feature-pages.css";

type Props = {
  routines: UserRoutine[];
  lastRun: RoutineRunResult | null;
  onRefresh: () => Promise<UserRoutine[]>;
  onSave: (request: SaveRoutineRequest) => Promise<UserRoutine>;
  onDelete: (routineId: string) => Promise<void>;
  onRun: (routineId: string) => Promise<RoutineRunResult>;
};

type DraftStep =
  | { type: "launchApp"; value: string }
  | { type: "switchToApp"; value: string }
  | { type: "directorPreset"; value: string }
  | { type: "wait"; value: string };

const APP_OPTIONS = [
  "obs",
  "brave",
  "chrome",
  "file explorer",
  "windows terminal",
  "notepad",
  "calculator",
];

function draftFromStep(step: RoutineStep): DraftStep {
  switch (step.type) {
    case "launchApp":
      return { type: "launchApp", value: step.app };
    case "switchToApp":
      return { type: "switchToApp", value: step.app };
    case "directorPreset":
      return { type: "directorPreset", value: step.preset };
    case "wait":
      return { type: "wait", value: String(step.milliseconds) };
  }
}

function stepFromDraft(step: DraftStep): RoutineStep | null {
  const value = step.value.trim();

  switch (step.type) {
    case "launchApp":
      return value ? { type: "launchApp", app: value } : null;
    case "switchToApp":
      return value ? { type: "switchToApp", app: value } : null;
    case "directorPreset":
      return value ? { type: "directorPreset", preset: value } : null;
    case "wait": {
      const milliseconds = Number(value);
      return Number.isFinite(milliseconds) && milliseconds >= 0
        ? { type: "wait", milliseconds: Math.round(milliseconds) }
        : null;
    }
  }
}

function stepLabel(step: RoutineStep) {
  switch (step.type) {
    case "launchApp":
      return `Launch ${step.app}`;
    case "switchToApp":
      return `Switch to ${step.app}`;
    case "directorPreset":
      return `Director preset: ${step.preset}`;
    case "wait":
      return `Wait ${step.milliseconds} ms`;
  }
}

export default function Tasks({
  routines,
  lastRun,
  onRefresh,
  onSave,
  onDelete,
  onRun,
}: Props) {
  const [editingId, setEditingId] = useState<string | undefined>();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [aliases, setAliases] = useState("");
  const [steps, setSteps] = useState<DraftStep[]>([]);
  const [newStepType, setNewStepType] =
    useState<DraftStep["type"]>("launchApp");
  const [newStepValue, setNewStepValue] = useState("");
  const [busy, setBusy] = useState(false);

  const editing = useMemo(
    () => routines.find((routine) => routine.id === editingId) ?? null,
    [editingId, routines],
  );

  function resetEditor() {
    setEditingId(undefined);
    setName("");
    setDescription("");
    setAliases("");
    setSteps([]);
    setNewStepType("launchApp");
    setNewStepValue("");
  }

  function editRoutine(routine: UserRoutine) {
    setEditingId(routine.id);
    setName(routine.name);
    setDescription(routine.description);
    setAliases(routine.aliases.join(", "));
    setSteps(routine.steps.map(draftFromStep));
  }

  function addStep() {
    const draft: DraftStep = { type: newStepType, value: newStepValue };
    if (!stepFromDraft(draft)) return;
    setSteps((current) => [...current, draft]);
    setNewStepValue("");
  }

  async function submit(event: FormEvent) {
    event.preventDefault();

    const resolvedSteps = steps
      .map(stepFromDraft)
      .filter((step): step is RoutineStep => Boolean(step));

    if (!name.trim() || resolvedSteps.length === 0) return;

    setBusy(true);
    try {
      await onSave({
        id: editingId,
        name: name.trim(),
        description: description.trim(),
        aliases: aliases
          .split(",")
          .map((alias) => alias.trim())
          .filter(Boolean),
        steps: resolvedSteps,
      });
      resetEditor();
    } finally {
      setBusy(false);
    }
  }

  async function removeRoutine(routineId: string) {
    setBusy(true);
    try {
      await onDelete(routineId);
      if (editingId === routineId) resetEditor();
    } finally {
      setBusy(false);
    }
  }

  async function runRoutine(routineId: string) {
    setBusy(true);
    try {
      await onRun(routineId);
    } finally {
      setBusy(false);
    }
  }

  const targetPlaceholder =
    newStepType === "directorPreset"
      ? "Prepare Match"
      : newStepType === "wait"
        ? "1000"
        : "brave";

  return (
    <section className="feature-page">
      <header className="feature-hero compact">
        <div>
          <span className="feature-kicker">AURA ROUTINES</span>
          <h2>Teach AURA repeatable workflows.</h2>
          <p>
            Build local multi-step routines from trusted AURA actions, then run
            them by name or alias from Chat.
          </p>
        </div>
      </header>

      <div className="routine-layout">
        <form className="feature-section routine-editor" onSubmit={submit}>
          <div className="feature-section-heading">
            <div>
              <span className="feature-kicker">
                {editing ? "EDIT ROUTINE" : "NEW ROUTINE"}
              </span>
              <strong>
                {editing ? editing.name : "Create a reusable workflow."}
              </strong>
            </div>
            {editing && (
              <button
                type="button"
                className="feature-secondary-button"
                onClick={resetEditor}
              >
                Cancel
              </button>
            )}
          </div>

          <label className="routine-field">
            <span>Name</span>
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Start Editing"
              maxLength={64}
            />
          </label>

          <label className="routine-field">
            <span>Description</span>
            <input
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="Prepare my editing workspace"
              maxLength={240}
            />
          </label>

          <label className="routine-field">
            <span>Aliases</span>
            <input
              value={aliases}
              onChange={(event) => setAliases(event.target.value)}
              placeholder="editing mode, prepare editing"
            />
          </label>

          <div className="routine-builder">
            <div className="routine-builder-controls">
              <select
                value={newStepType}
                onChange={(event) =>
                  setNewStepType(event.target.value as DraftStep["type"])
                }
              >
                <option value="launchApp">Launch app</option>
                <option value="switchToApp">Switch to app</option>
                <option value="directorPreset">Director preset</option>
                <option value="wait">Wait</option>
              </select>

              {newStepType === "launchApp" ||
              newStepType === "switchToApp" ? (
                <select
                  value={newStepValue}
                  onChange={(event) => setNewStepValue(event.target.value)}
                >
                  <option value="">Choose app…</option>
                  {APP_OPTIONS.map((app) => (
                    <option value={app} key={app}>
                      {app}
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  value={newStepValue}
                  onChange={(event) => setNewStepValue(event.target.value)}
                  placeholder={targetPlaceholder}
                  inputMode={newStepType === "wait" ? "numeric" : undefined}
                />
              )}

              <button
                type="button"
                className="feature-secondary-button"
                onClick={addStep}
              >
                Add step
              </button>
            </div>

            {steps.length === 0 ? (
              <div className="feature-empty">Add at least one routine step.</div>
            ) : (
              <div className="routine-steps">
                {steps.map((step, index) => {
                  const resolved = stepFromDraft(step);
                  return (
                    <article key={`${index}-${step.type}-${step.value}`}>
                      <span>{index + 1}</span>
                      <strong>
                        {resolved ? stepLabel(resolved) : "Invalid step"}
                      </strong>
                      <div>
                        <button
                          type="button"
                          disabled={index === 0}
                          onClick={() =>
                            setSteps((current) => {
                              const next = [...current];
                              [next[index - 1], next[index]] = [
                                next[index],
                                next[index - 1],
                              ];
                              return next;
                            })
                          }
                        >
                          ↑
                        </button>
                        <button
                          type="button"
                          disabled={index === steps.length - 1}
                          onClick={() =>
                            setSteps((current) => {
                              const next = [...current];
                              [next[index], next[index + 1]] = [
                                next[index + 1],
                                next[index],
                              ];
                              return next;
                            })
                          }
                        >
                          ↓
                        </button>
                        <button
                          type="button"
                          onClick={() =>
                            setSteps((current) =>
                              current.filter((_, itemIndex) => itemIndex !== index),
                            )
                          }
                        >
                          Remove
                        </button>
                      </div>
                    </article>
                  );
                })}
              </div>
            )}
          </div>

          <button
            type="submit"
            className="feature-primary-button"
            disabled={busy || !name.trim() || steps.length === 0}
          >
            {editing ? "Save changes" : "Create routine"}
          </button>
        </form>

        <div className="feature-section routine-library">
          <div className="feature-section-heading">
            <div>
              <span className="feature-kicker">SAVED ROUTINES</span>
              <strong>{routines.length} local routine(s)</strong>
            </div>
            <button
              type="button"
              className="feature-secondary-button"
              onClick={() => void onRefresh()}
              disabled={busy}
            >
              Refresh
            </button>
          </div>

          {routines.length === 0 ? (
            <div className="feature-empty">No routines saved yet.</div>
          ) : (
            <div className="routine-list">
              {routines.map((routine) => (
                <article key={routine.id}>
                  <div className="routine-card-copy">
                    <span>{routine.steps.length} STEPS</span>
                    <strong>{routine.name}</strong>
                    <p>
                      {routine.description ||
                        routine.aliases.join(" · ") ||
                        "No description"}
                    </p>
                  </div>
                  <div className="routine-card-actions">
                    <button
                      type="button"
                      onClick={() => void runRoutine(routine.id)}
                      disabled={busy}
                    >
                      Run
                    </button>
                    <button
                      type="button"
                      onClick={() => editRoutine(routine)}
                      disabled={busy}
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      onClick={() => void removeRoutine(routine.id)}
                      disabled={busy}
                    >
                      Delete
                    </button>
                  </div>
                </article>
              ))}
            </div>
          )}

          {lastRun && (
            <div className="routine-last-run">
              <span>LAST RUN</span>
              <strong>
                {lastRun.routineName} · {lastRun.success ? "Completed" : "Failed"}
              </strong>
              <small>
                {lastRun.completedSteps}/{lastRun.totalSteps} steps
                {lastRun.error ? ` · ${lastRun.error}` : ""}
              </small>
            </div>
          )}
        </div>
      </div>

      <div className="feature-note">
        <strong>Local and deterministic.</strong>
        <span>
          Saved routine steps are explicit AURA actions executed by the permission
          engine. For multi-step goals, reusable Actions and scheduled background
          work, use the Agents workspace.
        </span>
      </div>
    </section>
  );
}
