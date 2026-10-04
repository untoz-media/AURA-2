import { FormEvent, useMemo, useState } from "react";
import type {
  AgentPlan,
  AgentRun,
  AgentSnapshot,
  AgentStep,
  AuraAutomation,
  AutomationEvent,
  AutomationTrigger,
  DirectorPreset,
  SaveAuraActionRequest,
  SaveAutomationRequest,
  SavedAuraAction,
  UserRoutine,
} from "./bridge/types";
import "./feature-pages.css";
import "./agents.css";

type Props = {
  plan: AgentPlan | null;
  runs: AgentSnapshot;
  actions: SavedAuraAction[];
  automations: AuraAutomation[];
  automationEvent: AutomationEvent | null;
  routines: UserRoutine[];
  directorPresets: DirectorPreset[];
  onPlan: (goal: string) => Promise<AgentPlan>;
  onPlanClear: () => void;
  onRunPlan: (plan: AgentPlan, approved: boolean) => Promise<AgentRun>;
  onPauseRun: (runId: string, paused: boolean) => Promise<AgentRun>;
  onCancelRun: (runId: string) => Promise<AgentRun>;
  onActionSave: (request: SaveAuraActionRequest) => Promise<SavedAuraAction>;
  onActionDelete: (actionId: string) => Promise<void>;
  onActionRun: (actionId: string) => Promise<string>;
  onAutomationSave: (request: SaveAutomationRequest) => Promise<AuraAutomation>;
  onAutomationDelete: (automationId: string) => Promise<void>;
  onAutomationEnabled: (
    automationId: string,
    enabled: boolean,
  ) => Promise<AuraAutomation>;
};

type ActionKind =
  | "launchApp"
  | "switchToApp"
  | "runRoutine"
  | "directorPreset"
  | "wait";

type TriggerKind = AutomationTrigger["type"];

const APP_OPTIONS = [
  "obs",
  "brave",
  "chrome",
  "file explorer",
  "windows terminal",
  "notepad",
  "calculator",
];

function stepLabel(step: AgentStep) {
  switch (step.type) {
    case "launchApp":
      return `Launch ${step.app}`;
    case "switchToApp":
      return `Switch to ${step.app}`;
    case "runRoutine":
      return `Run routine: ${step.routine}`;
    case "directorPreset":
      return `Director preset: ${step.preset}`;
    case "wait":
      return `Wait ${step.milliseconds} ms`;
    case "savedAction":
      return `Saved Action: ${step.action}`;
  }
}

function triggerLabel(trigger: AutomationTrigger) {
  switch (trigger.type) {
    case "startup":
      return "When AURA starts";
    case "interval":
      return `Every ${trigger.everyMinutes} min`;
    case "atTime":
      return `${trigger.repeatDaily ? "Daily from" : "Once at"} ${new Date(
        trigger.runAtMs,
      ).toLocaleString()}`;
    case "appFocused":
      return `When ${trigger.app} gets focus`;
  }
}

function makeStep(
  kind: ActionKind,
  value: string,
): AgentStep | null {
  const trimmed = value.trim();
  if (!trimmed) return null;

  switch (kind) {
    case "launchApp":
      return { type: "launchApp", app: trimmed };
    case "switchToApp":
      return { type: "switchToApp", app: trimmed };
    case "runRoutine":
      return { type: "runRoutine", routine: trimmed };
    case "directorPreset":
      return { type: "directorPreset", preset: trimmed };
    case "wait": {
      const milliseconds = Number(trimmed);
      return Number.isFinite(milliseconds) && milliseconds >= 0
        ? { type: "wait", milliseconds: Math.round(milliseconds) }
        : null;
    }
  }
}

export default function Agents({
  plan,
  runs,
  actions,
  automations,
  automationEvent,
  routines,
  directorPresets,
  onPlan,
  onPlanClear,
  onRunPlan,
  onPauseRun,
  onCancelRun,
  onActionSave,
  onActionDelete,
  onActionRun,
  onAutomationSave,
  onAutomationDelete,
  onAutomationEnabled,
}: Props) {
  const [goal, setGoal] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [actionName, setActionName] = useState("");
  const [actionDescription, setActionDescription] = useState("");
  const [actionAliases, setActionAliases] = useState("");
  const [actionKind, setActionKind] = useState<ActionKind>("launchApp");
  const [actionValue, setActionValue] = useState("brave");

  const [automationName, setAutomationName] = useState("");
  const [automationActionId, setAutomationActionId] = useState("");
  const [triggerKind, setTriggerKind] = useState<TriggerKind>("startup");
  const [intervalMinutes, setIntervalMinutes] = useState("30");
  const [atTime, setAtTime] = useState("");
  const [repeatDaily, setRepeatDaily] = useState(false);
  const [focusedApp, setFocusedApp] = useState("brave");

  const activeRuns = useMemo(
    () =>
      runs.runs.filter((run) =>
        ["queued", "running", "paused"].includes(run.state),
      ),
    [runs.runs],
  );

  const recentRuns = useMemo(
    () =>
      runs.runs.filter((run) =>
        ["completed", "failed", "cancelled"].includes(run.state),
      ),
    [runs.runs],
  );

  async function buildPlan(event: FormEvent) {
    event.preventDefault();
    if (!goal.trim()) return;
    setBusy("plan");
    setError(null);
    try {
      await onPlan(goal.trim());
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(null);
    }
  }

  async function runPlan() {
    if (!plan) return;
    setBusy("run-plan");
    setError(null);
    try {
      await onRunPlan(plan, plan.requiresConfirmation);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(null);
    }
  }

  async function saveAction(event: FormEvent) {
    event.preventDefault();
    const step = makeStep(actionKind, actionValue);
    if (!actionName.trim() || !step) return;

    setBusy("action-save");
    setError(null);
    try {
      await onActionSave({
        name: actionName.trim(),
        description: actionDescription.trim(),
        aliases: actionAliases
          .split(",")
          .map((alias) => alias.trim())
          .filter(Boolean),
        step,
      });
      setActionName("");
      setActionDescription("");
      setActionAliases("");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(null);
    }
  }

  function buildTrigger(): AutomationTrigger | null {
    switch (triggerKind) {
      case "startup":
        return { type: "startup" };
      case "interval": {
        const everyMinutes = Number(intervalMinutes);
        return Number.isFinite(everyMinutes) && everyMinutes >= 1
          ? { type: "interval", everyMinutes: Math.round(everyMinutes) }
          : null;
      }
      case "atTime": {
        const parsed = new Date(atTime).getTime();
        return Number.isFinite(parsed) && parsed > Date.now()
          ? { type: "atTime", runAtMs: parsed, repeatDaily }
          : null;
      }
      case "appFocused":
        return { type: "appFocused", app: focusedApp };
    }
  }

  async function saveAutomation(event: FormEvent) {
    event.preventDefault();
    const trigger = buildTrigger();
    if (!automationName.trim() || !automationActionId || !trigger) return;

    setBusy("automation-save");
    setError(null);
    try {
      await onAutomationSave({
        name: automationName.trim(),
        enabled: true,
        actionId: automationActionId,
        trigger,
      });
      setAutomationName("");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(null);
    }
  }

  const actionTarget =
    actionKind === "runRoutine"
      ? routines.map((routine) => routine.name)
      : actionKind === "directorPreset"
        ? directorPresets.map((preset) => preset.name)
        : actionKind === "launchApp" || actionKind === "switchToApp"
          ? APP_OPTIONS
          : [];

  return (
    <section className="feature-page agents-page">
      <header className="feature-hero agents-hero">
        <div>
          <span className="feature-kicker">AURA AGENTS</span>
          <h2>Give AURA a goal, not a sequence of clicks.</h2>
          <p>
            The local planner proposes a bounded plan. AURA Core validates every
            step, classifies permissions and executes only supported Actions.
          </p>
        </div>
        <div className="agents-status-card">
          <span>{activeRuns.length > 0 ? "AGENT ACTIVE" : "AGENTS READY"}</span>
          <strong>{activeRuns.length}</strong>
          <small>active run{activeRuns.length === 1 ? "" : "s"}</small>
        </div>
      </header>

      {error && <div className="agents-error">{error}</div>}

      <div className="agents-grid">
        <article className="agents-panel planner-panel">
          <div className="agents-panel-heading">
            <div>
              <span className="feature-kicker">PLANNER</span>
              <h3>What should AURA accomplish?</h3>
            </div>
            <span className="feature-badge">Local model</span>
          </div>

          <form onSubmit={buildPlan} className="agent-goal-form">
            <textarea
              value={goal}
              onChange={(event) => setGoal(event.target.value)}
              placeholder="Example: Prepare my editing workspace by opening Brave and OBS, then run my Prepare Match routine."
              maxLength={1200}
            />
            <button
              className="feature-primary-button"
              type="submit"
              disabled={busy !== null || !goal.trim()}
            >
              {busy === "plan" ? "Planning locally…" : "Create plan"}
            </button>
          </form>

          {plan ? (
            <div className="agent-plan-card">
              <div className="agent-plan-summary">
                <div>
                  <span>PROPOSED PLAN</span>
                  <strong>{plan.summary}</strong>
                </div>
                <span className={`permission-pill ${plan.highestPermission}`}>
                  {plan.highestPermission}
                </span>
              </div>

              <ol className="agent-plan-steps">
                {plan.steps.map((step, index) => (
                  <li key={`${plan.id}-${index}`}>
                    <span>{index + 1}</span>
                    <div>
                      <strong>{stepLabel(step)}</strong>
                      <small>{step.type}</small>
                    </div>
                  </li>
                ))}
              </ol>

              {plan.blockedByPolicy ? (
                <div className="agents-warning">
                  This plan contains an Action blocked by the current permission
                  policy. Change that permission before it can run.
                </div>
              ) : plan.requiresConfirmation ? (
                <div className="agents-warning">
                  This plan contains an Action whose current permission policy
                  requires explicit approval.
                </div>
              ) : null}

              <div className="agents-actions-row">
                <button
                  type="button"
                  className="feature-primary-button"
                  disabled={busy !== null || plan.blockedByPolicy}
                  onClick={() => void runPlan()}
                >
                  {plan.blockedByPolicy
                    ? "Blocked by permissions"
                    : plan.requiresConfirmation
                      ? "Approve & run plan"
                      : "Run plan"}
                </button>
                <button
                  type="button"
                  className="feature-secondary-button"
                  onClick={onPlanClear}
                >
                  Discard plan
                </button>
              </div>
            </div>
          ) : (
            <div className="agents-empty">
              <strong>No plan yet.</strong>
              <span>
                Planning is isolated from your normal chat history and cannot
                create unsupported actions.
              </span>
            </div>
          )}
        </article>

        <article className="agents-panel">
          <div className="agents-panel-heading">
            <div>
              <span className="feature-kicker">EXECUTION</span>
              <h3>Background task status</h3>
            </div>
            <span className="feature-badge">{runs.runs.length} runs</span>
          </div>

          {activeRuns.length === 0 ? (
            <div className="agents-empty">
              <strong>No Agent is running.</strong>
              <span>Active runs will expose their current step here.</span>
            </div>
          ) : (
            <div className="agent-run-list">
              {activeRuns.map((run) => (
                <article className="agent-run-card active" key={run.id}>
                  <div className="agent-run-top">
                    <div>
                      <span>{run.state.toUpperCase()}</span>
                      <strong>{run.goal}</strong>
                    </div>
                    <em>
                      {run.completedSteps}/{run.totalSteps}
                    </em>
                  </div>
                  <div className="agent-progress-track">
                    <span
                      style={{
                        width: `${
                          run.totalSteps
                            ? (run.completedSteps / run.totalSteps) * 100
                            : 0
                        }%`,
                      }}
                    />
                  </div>
                  {run.currentStep != null && (
                    <small>
                      Current step: {run.currentStep + 1} of {run.totalSteps}
                    </small>
                  )}
                  <div className="agents-actions-row">
                    <button
                      type="button"
                      className="feature-secondary-button"
                      onClick={() =>
                        void onPauseRun(run.id, run.state !== "paused")
                      }
                    >
                      {run.state === "paused" ? "Resume" : "Pause"}
                    </button>
                    <button
                      type="button"
                      className="feature-secondary-button"
                      onClick={() => void onCancelRun(run.id)}
                    >
                      Cancel
                    </button>
                  </div>
                </article>
              ))}
            </div>
          )}

          {recentRuns.length > 0 && (
            <div className="agent-history">
              <span className="feature-kicker">RECENT RUNS</span>
              {recentRuns.slice(0, 5).map((run) => (
                <article key={run.id}>
                  <div>
                    <strong>{run.goal}</strong>
                    <span>{run.state}</span>
                  </div>
                  <small>
                    {run.completedSteps}/{run.totalSteps} steps
                    {run.error ? ` · ${run.error}` : ""}
                  </small>
                </article>
              ))}
            </div>
          )}
        </article>
      </div>

      <div className="agents-grid lower">
        <form className="agents-panel" onSubmit={saveAction}>
          <div className="agents-panel-heading">
            <div>
              <span className="feature-kicker">REUSABLE AURA ACTIONS</span>
              <h3>Save trusted building blocks.</h3>
            </div>
            <span className="feature-badge">{actions.length} saved</span>
          </div>

          <div className="agent-form-grid">
            <label>
              <span>Name</span>
              <input
                value={actionName}
                onChange={(event) => setActionName(event.target.value)}
                placeholder="Open production workspace"
              />
            </label>
            <label>
              <span>Description</span>
              <input
                value={actionDescription}
                onChange={(event) => setActionDescription(event.target.value)}
                placeholder="Reusable action for Agents and Automations"
              />
            </label>
            <label>
              <span>Aliases</span>
              <input
                value={actionAliases}
                onChange={(event) => setActionAliases(event.target.value)}
                placeholder="production, studio"
              />
            </label>
            <label>
              <span>Action type</span>
              <select
                value={actionKind}
                onChange={(event) => {
                  const kind = event.target.value as ActionKind;
                  setActionKind(kind);
                  setActionValue(
                    kind === "wait"
                      ? "1000"
                      : kind === "runRoutine"
                        ? routines[0]?.name ?? ""
                        : kind === "directorPreset"
                          ? directorPresets[0]?.name ?? ""
                          : "brave",
                  );
                }}
              >
                <option value="launchApp">Launch app</option>
                <option value="switchToApp">Switch to app</option>
                <option value="runRoutine">Run routine</option>
                <option value="directorPreset">Director preset</option>
                <option value="wait">Wait</option>
              </select>
            </label>
            <label className="agent-form-wide">
              <span>Target</span>
              {actionTarget.length > 0 ? (
                <select
                  value={actionValue}
                  onChange={(event) => setActionValue(event.target.value)}
                >
                  {actionTarget.map((value) => (
                    <option value={value} key={value}>
                      {value}
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  type="number"
                  value={actionValue}
                  onChange={(event) => setActionValue(event.target.value)}
                  placeholder="1000"
                />
              )}
            </label>
          </div>

          <button
            type="submit"
            className="feature-primary-button"
            disabled={busy !== null || !actionName.trim()}
          >
            Save AURA Action
          </button>

          <div className="saved-action-list">
            {actions.map((action) => (
              <article key={action.id}>
                <div>
                  <strong>{action.name}</strong>
                  <span>{stepLabel(action.step)}</span>
                </div>
                <div className="agents-actions-row">
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() => void onActionRun(action.id)}
                  >
                    Run
                  </button>
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() => void onActionDelete(action.id)}
                  >
                    Delete
                  </button>
                </div>
              </article>
            ))}
          </div>
        </form>

        <form className="agents-panel" onSubmit={saveAutomation}>
          <div className="agents-panel-heading">
            <div>
              <span className="feature-kicker">AUTOMATIONS</span>
              <h3>Run trusted Actions when something happens.</h3>
            </div>
            <span className="feature-badge">
              {automations.filter((item) => item.enabled).length} enabled
            </span>
          </div>

          <div className="agent-form-grid">
            <label>
              <span>Name</span>
              <input
                value={automationName}
                onChange={(event) => setAutomationName(event.target.value)}
                placeholder="Open studio at startup"
              />
            </label>
            <label>
              <span>AURA Action</span>
              <select
                value={automationActionId}
                onChange={(event) => setAutomationActionId(event.target.value)}
              >
                <option value="">Choose an Action</option>
                {actions.map((action) => (
                  <option value={action.id} key={action.id}>
                    {action.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>Trigger</span>
              <select
                value={triggerKind}
                onChange={(event) =>
                  setTriggerKind(event.target.value as TriggerKind)
                }
              >
                <option value="startup">AURA startup</option>
                <option value="interval">Interval</option>
                <option value="atTime">Date / time</option>
                <option value="appFocused">App focused</option>
              </select>
            </label>

            {triggerKind === "interval" && (
              <label>
                <span>Every (minutes)</span>
                <input
                  type="number"
                  min={1}
                  max={10080}
                  value={intervalMinutes}
                  onChange={(event) => setIntervalMinutes(event.target.value)}
                />
              </label>
            )}

            {triggerKind === "atTime" && (
              <>
                <label>
                  <span>Date / time</span>
                  <input
                    type="datetime-local"
                    value={atTime}
                    onChange={(event) => setAtTime(event.target.value)}
                  />
                </label>
                <label className="agent-checkbox">
                  <input
                    type="checkbox"
                    checked={repeatDaily}
                    onChange={(event) => setRepeatDaily(event.target.checked)}
                  />
                  <span>Repeat every 24 hours</span>
                </label>
              </>
            )}

            {triggerKind === "appFocused" && (
              <label>
                <span>Application</span>
                <select
                  value={focusedApp}
                  onChange={(event) => setFocusedApp(event.target.value)}
                >
                  {APP_OPTIONS.map((app) => (
                    <option value={app} key={app}>
                      {app}
                    </option>
                  ))}
                </select>
              </label>
            )}
          </div>

          <button
            type="submit"
            className="feature-primary-button"
            disabled={
              busy !== null ||
              !automationName.trim() ||
              !automationActionId
            }
          >
            Create automation
          </button>

          {automationEvent && (
            <div className="automation-last-event">
              <span>{automationEvent.status}</span>
              <strong>{automationEvent.automationName}</strong>
              <small>{automationEvent.message}</small>
            </div>
          )}

          <div className="automation-list">
            {automations.map((automation) => (
              <article key={automation.id}>
                <div className="automation-main">
                  <span
                    className={`automation-dot ${automation.enabled ? "enabled" : ""}`}
                  />
                  <div>
                    <strong>{automation.name}</strong>
                    <span>{triggerLabel(automation.trigger)}</span>
                    {automation.lastResult && (
                      <small>{automation.lastResult}</small>
                    )}
                  </div>
                </div>
                <div className="agents-actions-row">
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() =>
                      void onAutomationEnabled(
                        automation.id,
                        !automation.enabled,
                      )
                    }
                  >
                    {automation.enabled ? "Disable" : "Enable"}
                  </button>
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() => void onAutomationDelete(automation.id)}
                  >
                    Delete
                  </button>
                </div>
              </article>
            ))}
          </div>

          <div className="agents-safety-note">
            Background automations execute saved Actions only. Sensitive
            routines/Director presets are blocked from unattended execution.
          </div>
        </form>
      </div>
    </section>
  );
}
