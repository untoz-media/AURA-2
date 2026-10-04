use crate::{
    computer::{
        app_launcher::{launch_app, AppTarget},
        window_manager::{current_app, switch_to_app},
    },
    integrations::{
        director::{
            find_director_preset_by_id, load_director_presets, preset_requires_sensitive_permission,
            run_director_preset,
        },
        obs::ObsController,
    },
    model_manager::ModelManager,
    model_runtime::ModelRuntime,
    permissions::{PermissionClass, PermissionDecision, PermissionPolicy},
    routines::{
        find_routine_by_id, list_routines, routine_requires_sensitive_permission, run_routine,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};

const ACTIONS_FILENAME: &str = "aura-actions.json";
const AUTOMATIONS_FILENAME: &str = "aura-automations.json";
const RUN_HISTORY_FILENAME: &str = "agent-runs.json";
const MAX_PLAN_STEPS: usize = 12;
const MAX_SAVED_ACTIONS: usize = 64;
const MAX_AUTOMATIONS: usize = 64;
const MAX_HISTORY: usize = 50;
const MAX_WAIT_MS: u64 = 30_000;
const AGENT_EVENT: &str = "aura:agent-event";
const AUTOMATION_EVENT: &str = "aura:automation-event";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AgentStep {
    LaunchApp { app: String },
    SwitchToApp { app: String },
    RunRoutine { routine: String },
    DirectorPreset { preset: String },
    Wait { milliseconds: u64 },
    SavedAction { action: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPlan {
    pub id: String,
    pub goal: String,
    pub summary: String,
    pub steps: Vec<AgentStep>,
    pub requires_confirmation: bool,
    pub blocked_by_policy: bool,
    pub highest_permission: PermissionClass,
    pub created_at_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlannedPayload {
    summary: String,
    steps: Vec<AgentStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStepResult {
    pub index: usize,
    pub label: String,
    pub status: String,
    pub attempts: u8,
    pub message: String,
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    pub id: String,
    pub plan_id: String,
    pub goal: String,
    pub state: String,
    pub current_step: Option<usize>,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub steps: Vec<AgentStepResult>,
    pub started_at_ms: u64,
    pub completed_at_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSnapshot {
    pub runs: Vec<AgentRun>,
    pub refreshed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedAuraAction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub step: AgentStep,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAuraActionRequest {
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub step: AgentStep,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AutomationTrigger {
    Startup,
    Interval { every_minutes: u64 },
    AtTime { run_at_ms: u64, repeat_daily: bool },
    AppFocused { app: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuraAutomation {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub action_id: String,
    pub trigger: AutomationTrigger,
    pub last_run_at_ms: Option<u64>,
    pub next_run_at_ms: Option<u64>,
    pub last_result: Option<String>,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAutomationRequest {
    pub id: Option<String>,
    pub name: String,
    pub enabled: bool,
    pub action_id: String,
    pub trigger: AutomationTrigger,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    pub run: AgentRun,
    pub message: String,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationEvent {
    pub automation_id: String,
    pub automation_name: String,
    pub status: String,
    pub message: String,
    pub timestamp_ms: u64,
}

#[derive(Clone)]
struct RunControl {
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
}

pub struct AgentEngine {
    runs: Arc<Mutex<HashMap<String, AgentRun>>>,
    controls: Arc<Mutex<HashMap<String, RunControl>>>,
    global_paused: Arc<AtomicBool>,
}

impl Default for AgentEngine {
    fn default() -> Self {
        Self {
            runs: Arc::new(Mutex::new(HashMap::new())),
            controls: Arc::new(Mutex::new(HashMap::new())),
            global_paused: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub struct AutomationScheduler {
    generation: AtomicU64,
    paused: AtomicBool,
    permission_policy: Mutex<PermissionPolicy>,
    startup_fired: Mutex<HashSet<String>>,
    last_external_process: Mutex<Option<String>>,
}

impl Default for AutomationScheduler {
    fn default() -> Self {
        Self {
            generation: AtomicU64::new(0),
            paused: AtomicBool::new(false),
            permission_policy: Mutex::new(PermissionPolicy::default()),
            startup_fired: Mutex::new(HashSet::new()),
            last_external_process: Mutex::new(None),
        }
    }
}

pub fn plan_goal(
    app: &AppHandle,
    manager: &ModelManager,
    runtime: &ModelRuntime,
    policy: &PermissionPolicy,
    goal: &str,
) -> Result<AgentPlan, String> {
    let goal = validate_text(goal, "Agent goal", 1200)?;
    let routines = list_routines(app).unwrap_or_default();
    let presets = load_director_presets(app);
    let actions = list_saved_actions(app).unwrap_or_default();

    let routine_names = routines
        .iter()
        .map(|routine| routine.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let preset_names = presets
        .iter()
        .map(|preset| preset.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let action_names = actions
        .iter()
        .map(|action| action.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let planner_prompt = format!(
        r#"You are AURA Agent Planner. Produce ONLY one JSON object, with no markdown and no commentary.

Goal:
{goal}

You may use only these step shapes:
{{"type":"launchApp","app":"brave"}}
{{"type":"switchToApp","app":"obs"}}
{{"type":"runRoutine","routine":"Exact routine name"}}
{{"type":"directorPreset","preset":"Exact preset name"}}
{{"type":"wait","milliseconds":1000}}
{{"type":"savedAction","action":"Exact saved action name"}}

Supported app aliases include: obs, brave, chrome, file explorer, windows terminal, notepad, calculator.
Available routines: {routine_names}
Available Director presets: {preset_names}
Available saved AURA Actions: {action_names}

Rules:
- Maximum {MAX_PLAN_STEPS} steps.
- Never invent an app, routine, preset or saved action that is not available.
- Prefer reusable routines/actions when they already match the goal.
- Use waits only when genuinely useful and never over {MAX_WAIT_MS} ms.
- Do not include mouse coordinates, arbitrary text typing, shell commands, file deletion, shutdown/restart, streaming start, purchases, messages, uploads or any unsupported action.
- The JSON shape MUST be:
{{"summary":"short explanation","steps":[...]}}
- If the goal cannot be safely represented with the allowed actions, return:
{{"summary":"Cannot safely plan this goal with the available AURA Actions.","steps":[]}}"#
    );

    let raw = runtime.generate_isolated(app, manager, &planner_prompt, None)?;
    let payload = parse_planned_payload(&raw)?;
    validate_plan_steps(app, &payload.steps)?;

    if payload.steps.is_empty() {
        return Err(payload.summary);
    }

    let highest_permission = highest_permission_for_steps(app, &payload.steps)?;
    let mut requires_confirmation = false;
    let mut blocked_by_policy = false;

    for step in &payload.steps {
        let permission = permission_for_step(app, step)?;
        match policy.decision_for(permission) {
            PermissionDecision::Ask => requires_confirmation = true,
            PermissionDecision::Never => blocked_by_policy = true,
            PermissionDecision::Allow => {}
        }
    }

    Ok(AgentPlan {
        id: format!("plan-{}", timestamp_ms()),
        goal,
        summary: validate_text(&payload.summary, "Plan summary", 500)?,
        steps: payload.steps,
        requires_confirmation,
        blocked_by_policy,
        highest_permission,
        created_at_ms: timestamp_ms(),
    })
}

impl AgentEngine {
    pub fn snapshot(&self, app: &AppHandle) -> Result<AgentSnapshot, String> {
        let active = self
            .runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let history = read_run_history(app).unwrap_or_default();

        let mut by_id = HashMap::<String, AgentRun>::new();
        for run in history.into_iter().chain(active) {
            by_id.insert(run.id.clone(), run);
        }

        let mut runs = by_id.into_values().collect::<Vec<_>>();
        runs.sort_by(|left, right| right.started_at_ms.cmp(&left.started_at_ms));
        runs.truncate(MAX_HISTORY);

        Ok(AgentSnapshot {
            runs,
            refreshed_at_ms: timestamp_ms(),
        })
    }

    pub fn start(
        &self,
        app: AppHandle,
        plan: AgentPlan,
        policy: PermissionPolicy,
        approved: bool,
    ) -> Result<AgentRun, String> {
        validate_plan_steps(&app, &plan.steps)?;
        validate_plan_permission(&app, &plan.steps, &policy, approved)?;

        let run = AgentRun {
            id: format!("run-{}", timestamp_ms()),
            plan_id: plan.id.clone(),
            goal: plan.goal.clone(),
            state: "queued".to_string(),
            current_step: None,
            completed_steps: 0,
            total_steps: plan.steps.len(),
            steps: Vec::new(),
            started_at_ms: timestamp_ms(),
            completed_at_ms: None,
            error: None,
        };

        let control = RunControl {
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
        };

        self.runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(run.id.clone(), run.clone());
        self.controls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(run.id.clone(), control.clone());

        emit_agent_event(&app, &run, "Agent queued.");

        let runs = Arc::clone(&self.runs);
        let controls = Arc::clone(&self.controls);
        let global_paused = Arc::clone(&self.global_paused);
        let run_id = run.id.clone();

        thread::spawn(move || {
            execute_agent_plan(
                &app,
                &runs,
                &controls,
                &global_paused,
                &run_id,
                &plan,
                &control,
            );
        });

        Ok(run)
    }

    pub fn pause(&self, run_id: &str, paused: bool) -> Result<AgentRun, String> {
        let controls = self
            .controls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let control = controls
            .get(run_id)
            .ok_or_else(|| "Agent run is not active.".to_string())?;
        control.paused.store(paused, Ordering::SeqCst);
        drop(controls);

        let mut runs = self
            .runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let run = runs
            .get_mut(run_id)
            .ok_or_else(|| "Agent run was not found.".to_string())?;

        if !matches!(run.state.as_str(), "completed" | "failed" | "cancelled") {
            run.state = if paused { "paused" } else { "running" }.to_string();
        }
        Ok(run.clone())
    }

    pub fn cancel(&self, run_id: &str) -> Result<AgentRun, String> {
        let controls = self
            .controls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let control = controls
            .get(run_id)
            .ok_or_else(|| "Agent run is not active.".to_string())?;
        control.cancelled.store(true, Ordering::SeqCst);
        control.paused.store(false, Ordering::SeqCst);
        drop(controls);

        let runs = self
            .runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        runs.get(run_id)
            .cloned()
            .ok_or_else(|| "Agent run was not found.".to_string())
    }

    pub fn set_global_paused(&self, app: &AppHandle, paused: bool) {
        self.global_paused.store(paused, Ordering::SeqCst);

        let controls = self
            .controls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        let snapshots = {
            let mut runs = self
                .runs
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut changed = Vec::new();

            for run in runs.values_mut() {
                if matches!(run.state.as_str(), "completed" | "failed" | "cancelled") {
                    continue;
                }

                let individually_paused = controls
                    .get(&run.id)
                    .is_some_and(|control| control.paused.load(Ordering::SeqCst));

                run.state = if paused || individually_paused {
                    "paused".to_string()
                } else {
                    "running".to_string()
                };
                changed.push(run.clone());
            }
            changed
        };

        for run in snapshots {
            emit_agent_event(
                app,
                &run,
                if paused {
                    "Agent paused because AURA is paused."
                } else {
                    "Agent resumed with AURA."
                },
            );
        }
    }
}

impl AutomationScheduler {
    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
    }

    pub fn set_permission_policy(&self, policy: PermissionPolicy) {
        *self
            .permission_policy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = policy;
    }

    pub fn start(&self, app: AppHandle) {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.startup_fired
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        *self
            .last_external_process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;

        let handle = app.clone();
        thread::spawn(move || loop {
            let scheduler = handle.state::<AutomationScheduler>();
            if scheduler.generation.load(Ordering::SeqCst) != generation {
                return;
            }

            if let Err(error) = scheduler.tick(&handle) {
                let _ = handle.emit(
                    AUTOMATION_EVENT,
                    AutomationEvent {
                        automation_id: "scheduler".to_string(),
                        automation_name: "Automation scheduler".to_string(),
                        status: "error".to_string(),
                        message: error,
                        timestamp_ms: timestamp_ms(),
                    },
                );
            }

            thread::sleep(Duration::from_secs(2));
        });
    }

    fn tick(&self, app: &AppHandle) -> Result<(), String> {
        let mut automations = read_automations(app)?;
        if automations.is_empty() {
            return Ok(());
        }

        let now = timestamp_ms();
        let foreground = current_app().ok().filter(|context| {
            context.process_id != std::process::id()
        });
        let current_process = foreground
            .as_ref()
            .map(|context| context.process_name.to_lowercase());
        let previous_process = self
            .last_external_process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let app_changed = current_process != previous_process;

        if app_changed {
            *self
                .last_external_process
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                current_process.clone();
        }

        let mut changed = false;

        for automation in &mut automations {
            if !automation.enabled {
                continue;
            }

            let due = match &automation.trigger {
                AutomationTrigger::Startup => {
                    let mut fired = self
                        .startup_fired
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    fired.insert(automation.id.clone())
                }
                AutomationTrigger::Interval { .. } => {
                    automation.next_run_at_ms.is_some_and(|next| now >= next)
                }
                AutomationTrigger::AtTime { run_at_ms, .. } => {
                    automation.next_run_at_ms.unwrap_or(*run_at_ms) <= now
                }
                AutomationTrigger::AppFocused { app: target } => {
                    app_changed
                        && foreground
                            .as_ref()
                            .is_some_and(|context| app_matches_context(target, &context.process_name))
                }
            };

            if !due {
                continue;
            }

            let result = if self.paused.load(Ordering::SeqCst) {
                Err("AURA is paused; background automation was skipped.".to_string())
            } else {
                let permission = saved_action_permission(app, &automation.action_id);
                let policy = self
                    .permission_policy
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .clone();

                match permission {
                    Ok(permission)
                        if policy.decision_for(permission) == PermissionDecision::Allow =>
                    {
                        execute_saved_action(app, &automation.action_id, true)
                    }
                    Ok(permission) => Err(format!(
                        "Background automation cannot run because {:?} permission is not Allow.",
                        permission
                    )),
                    Err(error) => Err(error),
                }
            };
            automation.last_run_at_ms = Some(now);
            automation.last_result = Some(match &result {
                Ok(message) => message.clone(),
                Err(error) => error.clone(),
            });
            automation.updated_at_ms = now;
            changed = true;

            match &automation.trigger {
                AutomationTrigger::Interval { every_minutes } => {
                    automation.next_run_at_ms = Some(
                        now.saturating_add(every_minutes.saturating_mul(60_000)),
                    );
                }
                AutomationTrigger::AtTime {
                    repeat_daily: true,
                    ..
                } => {
                    automation.next_run_at_ms =
                        Some(now.saturating_add(24 * 60 * 60 * 1000));
                }
                AutomationTrigger::AtTime {
                    repeat_daily: false,
                    ..
                } => {
                    automation.enabled = false;
                    automation.next_run_at_ms = None;
                }
                _ => {}
            }

            let (status, message) = match result {
                Ok(message) => ("completed", message),
                Err(error) => ("failed", error),
            };

            let _ = app.emit(
                AUTOMATION_EVENT,
                AutomationEvent {
                    automation_id: automation.id.clone(),
                    automation_name: automation.name.clone(),
                    status: status.to_string(),
                    message,
                    timestamp_ms: now,
                },
            );
        }

        if changed {
            write_automations(app, &automations)?;
        }

        Ok(())
    }
}

pub fn list_saved_actions(app: &AppHandle) -> Result<Vec<SavedAuraAction>, String> {
    read_actions(app)
}

pub fn save_action(
    app: &AppHandle,
    request: SaveAuraActionRequest,
) -> Result<SavedAuraAction, String> {
    let mut actions = read_actions(app)?;
    if request.id.is_none() && actions.len() >= MAX_SAVED_ACTIONS {
        return Err(format!("AURA supports at most {MAX_SAVED_ACTIONS} saved Actions."));
    }

    let name = validate_text(&request.name, "Action name", 80)?;
    let description = request.description.trim().to_string();
    if description.chars().count() > 300 {
        return Err("Action description is too long.".to_string());
    }
    validate_action_step(app, &request.step)?;

    let aliases = sanitize_aliases(&request.aliases, &name, 12)?;
    let requested_id = request.id.as_deref();

    for existing in &actions {
        if requested_id == Some(existing.id.as_str()) {
            continue;
        }
        if action_name_conflicts(existing, &name, &aliases) {
            return Err(format!(
                "Action name or alias conflicts with existing Action “{}”.",
                existing.name
            ));
        }
    }

    let id = match requested_id {
        Some(id) => {
            if !actions.iter().any(|action| action.id == id) {
                return Err("Saved AURA Action no longer exists.".to_string());
            }
            id.to_string()
        }
        None => format!("action-{}-{}", slugify(&name), timestamp_ms()),
    };

    let saved = SavedAuraAction {
        id: id.clone(),
        name,
        description,
        aliases,
        step: request.step,
        updated_at_ms: timestamp_ms(),
    };

    if let Some(index) = actions.iter().position(|action| action.id == id) {
        actions[index] = saved.clone();
    } else {
        actions.push(saved.clone());
    }

    actions.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    write_actions(app, &actions)?;
    Ok(saved)
}

pub fn delete_action(app: &AppHandle, action_id: &str) -> Result<(), String> {
    let automations = read_automations(app)?;
    if automations.iter().any(|automation| automation.action_id == action_id) {
        return Err("This AURA Action is still used by an automation.".to_string());
    }

    let mut actions = read_actions(app)?;
    let before = actions.len();
    actions.retain(|action| action.id != action_id);
    if actions.len() == before {
        return Err("Saved AURA Action was not found.".to_string());
    }
    write_actions(app, &actions)
}

pub fn run_saved_action(
    app: &AppHandle,
    action_id: &str,
    policy: &PermissionPolicy,
) -> Result<String, String> {
    let permission = saved_action_permission(app, action_id)?;
    if policy.decision_for(permission) == PermissionDecision::Never {
        return Err(format!(
            "Saved AURA Action is blocked by the current {:?} permission policy.",
            permission
        ));
    }
    execute_saved_action(app, action_id, false)
}

pub fn saved_action_permission(
    app: &AppHandle,
    action_id: &str,
) -> Result<PermissionClass, String> {
    let action = resolve_saved_action(app, action_id)
        .ok_or_else(|| "Saved AURA Action was not found.".to_string())?;
    permission_for_step(app, &action.step)
}

pub fn list_automations(app: &AppHandle) -> Result<Vec<AuraAutomation>, String> {
    read_automations(app)
}

pub fn save_automation(
    app: &AppHandle,
    request: SaveAutomationRequest,
) -> Result<AuraAutomation, String> {
    let mut automations = read_automations(app)?;
    if request.id.is_none() && automations.len() >= MAX_AUTOMATIONS {
        return Err(format!("AURA supports at most {MAX_AUTOMATIONS} automations."));
    }

    let name = validate_text(&request.name, "Automation name", 80)?;
    let actions = read_actions(app)?;
    if !actions.iter().any(|action| action.id == request.action_id) {
        return Err("Automation references an AURA Action that does not exist.".to_string());
    }
    validate_trigger(&request.trigger)?;

    let id = request
        .id
        .clone()
        .unwrap_or_else(|| format!("automation-{}-{}", slugify(&name), timestamp_ms()));

    if request.id.is_some() && !automations.iter().any(|automation| automation.id == id) {
        return Err("Automation no longer exists.".to_string());
    }

    if automations
        .iter()
        .any(|automation| automation.id != id && automation.name.eq_ignore_ascii_case(&name))
    {
        return Err("Another automation already uses this name.".to_string());
    }

    let now = timestamp_ms();
    let next_run_at_ms = initial_next_run(&request.trigger, now);

    let previous = automations.iter().find(|automation| automation.id == id);
    let saved = AuraAutomation {
        id: id.clone(),
        name,
        enabled: request.enabled,
        action_id: request.action_id,
        trigger: request.trigger,
        last_run_at_ms: previous.and_then(|automation| automation.last_run_at_ms),
        next_run_at_ms,
        last_result: previous.and_then(|automation| automation.last_result.clone()),
        updated_at_ms: now,
    };

    if let Some(index) = automations.iter().position(|automation| automation.id == id) {
        automations[index] = saved.clone();
    } else {
        automations.push(saved.clone());
    }

    automations.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    write_automations(app, &automations)?;
    Ok(saved)
}

pub fn delete_automation(app: &AppHandle, automation_id: &str) -> Result<(), String> {
    let mut automations = read_automations(app)?;
    let before = automations.len();
    automations.retain(|automation| automation.id != automation_id);
    if automations.len() == before {
        return Err("Automation was not found.".to_string());
    }
    write_automations(app, &automations)
}

pub fn set_automation_enabled(
    app: &AppHandle,
    automation_id: &str,
    enabled: bool,
) -> Result<AuraAutomation, String> {
    let mut automations = read_automations(app)?;
    let automation = automations
        .iter_mut()
        .find(|automation| automation.id == automation_id)
        .ok_or_else(|| "Automation was not found.".to_string())?;

    automation.enabled = enabled;
    automation.updated_at_ms = timestamp_ms();
    if enabled {
        automation.next_run_at_ms = initial_next_run(&automation.trigger, timestamp_ms());
    }

    let result = automation.clone();
    write_automations(app, &automations)?;
    Ok(result)
}

fn execute_agent_plan(
    app: &AppHandle,
    runs: &Arc<Mutex<HashMap<String, AgentRun>>>,
    controls: &Arc<Mutex<HashMap<String, RunControl>>>,
    global_paused: &Arc<AtomicBool>,
    run_id: &str,
    plan: &AgentPlan,
    control: &RunControl,
) {
    update_run(app, runs, run_id, |run| {
        run.state = "running".to_string();
    }, "Agent started.");

    for (index, step) in plan.steps.iter().enumerate() {
        while control.paused.load(Ordering::SeqCst)
            || global_paused.load(Ordering::SeqCst)
        {
            if control.cancelled.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(120));
        }

        if control.cancelled.load(Ordering::SeqCst) {
            finish_run(app, runs, controls, run_id, "cancelled", None);
            return;
        }

        update_run(app, runs, run_id, |run| {
            run.current_step = Some(index);
        }, &format!("Running step {} of {}: {}", index + 1, plan.steps.len(), step_label(step)));

        let max_attempts = if step_is_retryable(step) { 2 } else { 1 };
        let started_at_ms = timestamp_ms();
        let mut attempts = 0_u8;
        let mut outcome = Err("Agent step did not run.".to_string());

        while attempts < max_attempts {
            attempts += 1;
            outcome = execute_step(app, step, false);
            if outcome.is_ok() || attempts >= max_attempts {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }

        let completed_at_ms = timestamp_ms();
        match outcome {
            Ok(message) => {
                let result = AgentStepResult {
                    index,
                    label: step_label(step),
                    status: "completed".to_string(),
                    attempts,
                    message,
                    started_at_ms,
                    completed_at_ms,
                };
                update_run(app, runs, run_id, |run| {
                    run.steps.push(result);
                    run.completed_steps += 1;
                    run.current_step = None;
                }, &format!("Completed step {}.", index + 1));
            }
            Err(error) => {
                let result = AgentStepResult {
                    index,
                    label: step_label(step),
                    status: "failed".to_string(),
                    attempts,
                    message: error.clone(),
                    started_at_ms,
                    completed_at_ms,
                };
                update_run(app, runs, run_id, |run| {
                    run.steps.push(result);
                    run.current_step = None;
                    run.error = Some(error.clone());
                }, &format!("Agent failed at step {}: {error}", index + 1));
                finish_run(app, runs, controls, run_id, "failed", Some(error));
                return;
            }
        }
    }

    finish_run(app, runs, controls, run_id, "completed", None);
}

fn step_is_retryable(step: &AgentStep) -> bool {
    matches!(
        step,
        AgentStep::LaunchApp { .. } | AgentStep::SwitchToApp { .. }
    )
}

fn execute_step(app: &AppHandle, step: &AgentStep, background: bool) -> Result<String, String> {
    match step {
        AgentStep::LaunchApp { app: target } => {
            let target = resolve_app(target)?;
            launch_app(target)
                .map(|_| format!("Opened {}.", target.display_name()))
                .map_err(|error| error.to_string())
        }
        AgentStep::SwitchToApp { app: target } => {
            let target = resolve_app(target)?;
            switch_to_app(target)
                .map(|window| format!("Focused {}: {}.", target.display_name(), window.title))
                .map_err(|error| error.to_string())
        }
        AgentStep::RunRoutine { routine } => {
            let resolved = find_routine_by_id(app, routine)
                .or_else(|| {
                    list_routines(app).ok()?.into_iter().find(|item| {
                        item.name.eq_ignore_ascii_case(routine)
                            || item.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(routine))
                    })
                })
                .ok_or_else(|| format!("Routine “{routine}” was not found."))?;

            if background && routine_requires_sensitive_permission(app, &resolved) {
                return Err("Background automations cannot run sensitive routines.".to_string());
            }

            let obs = app.state::<ObsController>();
            let result = tauri::async_runtime::block_on(run_routine(app, &obs, &resolved));
            if result.success {
                Ok(format!(
                    "Routine {} completed ({} steps).",
                    result.routine_name, result.completed_steps
                ))
            } else {
                Err(result.error.unwrap_or_else(|| "Routine failed.".to_string()))
            }
        }
        AgentStep::DirectorPreset { preset } => {
            let resolved = find_director_preset_by_id(app, preset)
                .or_else(|| {
                    load_director_presets(app).into_iter().find(|item| {
                        item.name.eq_ignore_ascii_case(preset)
                            || item.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(preset))
                    })
                })
                .ok_or_else(|| format!("Director preset “{preset}” was not found."))?;

            if background && preset_requires_sensitive_permission(&resolved) {
                return Err(
                    "Background automations cannot run a sensitive Director preset.".to_string(),
                );
            }

            let obs = app.state::<ObsController>();
            let result = tauri::async_runtime::block_on(run_director_preset(&obs, &resolved));
            if result.success {
                Ok(format!(
                    "Director preset {} completed ({} steps).",
                    result.preset_name, result.completed_steps
                ))
            } else {
                Err(result
                    .error
                    .unwrap_or_else(|| "Director preset failed.".to_string()))
            }
        }
        AgentStep::Wait { milliseconds } => {
            if *milliseconds > MAX_WAIT_MS {
                return Err(format!(
                    "Agent waits cannot exceed {} seconds.",
                    MAX_WAIT_MS / 1000
                ));
            }
            thread::sleep(Duration::from_millis(*milliseconds));
            Ok(format!("Waited {milliseconds} ms."))
        }
        AgentStep::SavedAction { action } => {
            let saved = resolve_saved_action(app, action)
                .ok_or_else(|| format!("Saved AURA Action “{action}” was not found."))?;
            execute_step(app, &saved.step, background)
        }
    }
}

fn execute_saved_action(
    app: &AppHandle,
    action_id: &str,
    background: bool,
) -> Result<String, String> {
    let action = resolve_saved_action(app, action_id)
        .ok_or_else(|| "Saved AURA Action was not found.".to_string())?;

    if background {
        let permission = permission_for_step(app, &action.step)?;
        if !matches!(permission, PermissionClass::Read | PermissionClass::Act) {
            return Err("Background automations can only execute Read/Act Actions.".to_string());
        }
    }

    execute_step(app, &action.step, background)
}

fn validate_plan_permission(
    app: &AppHandle,
    steps: &[AgentStep],
    policy: &PermissionPolicy,
    approved: bool,
) -> Result<(), String> {
    for step in steps {
        let permission = permission_for_step(app, step)?;
        match policy.decision_for(permission) {
            PermissionDecision::Never => {
                return Err(format!(
                    "Agent step “{}” is blocked by the current {:?} permission policy.",
                    step_label(step), permission
                ))
            }
            PermissionDecision::Ask if !approved => {
                return Err(
                    "This Agent plan requires confirmation. Review the plan and run it with approval."
                        .to_string(),
                )
            }
            _ => {}
        }
    }
    Ok(())
}

fn permission_for_step(app: &AppHandle, step: &AgentStep) -> Result<PermissionClass, String> {
    Ok(match step {
        AgentStep::LaunchApp { .. } | AgentStep::SwitchToApp { .. } => PermissionClass::Act,
        AgentStep::Wait { .. } => PermissionClass::Read,
        AgentStep::RunRoutine { routine } => {
            let resolved = find_routine_by_id(app, routine)
                .or_else(|| {
                    list_routines(app).ok()?.into_iter().find(|item| {
                        item.name.eq_ignore_ascii_case(routine)
                            || item.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(routine))
                    })
                })
                .ok_or_else(|| format!("Routine “{routine}” was not found."))?;
            if routine_requires_sensitive_permission(app, &resolved) {
                PermissionClass::Sensitive
            } else {
                PermissionClass::Act
            }
        }
        AgentStep::DirectorPreset { preset } => {
            let resolved = find_director_preset_by_id(app, preset)
                .or_else(|| {
                    load_director_presets(app).into_iter().find(|item| {
                        item.name.eq_ignore_ascii_case(preset)
                            || item.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(preset))
                    })
                })
                .ok_or_else(|| format!("Director preset “{preset}” was not found."))?;
            if preset_requires_sensitive_permission(&resolved) {
                PermissionClass::Sensitive
            } else {
                PermissionClass::Act
            }
        }
        AgentStep::SavedAction { action } => {
            let saved = resolve_saved_action(app, action)
                .ok_or_else(|| format!("Saved AURA Action “{action}” was not found."))?;
            permission_for_step(app, &saved.step)?
        }
    })
}

fn highest_permission_for_steps(
    app: &AppHandle,
    steps: &[AgentStep],
) -> Result<PermissionClass, String> {
    let mut highest = PermissionClass::Read;
    for step in steps {
        let permission = permission_for_step(app, step)?;
        if permission_rank(permission) > permission_rank(highest) {
            highest = permission;
        }
    }
    Ok(highest)
}

fn permission_rank(permission: PermissionClass) -> u8 {
    match permission {
        PermissionClass::Read => 0,
        PermissionClass::Act => 1,
        PermissionClass::Modify => 2,
        PermissionClass::Sensitive => 3,
        PermissionClass::Destructive => 4,
    }
}

fn validate_plan_steps(app: &AppHandle, steps: &[AgentStep]) -> Result<(), String> {
    if steps.is_empty() {
        return Err("Agent plan has no executable steps.".to_string());
    }
    if steps.len() > MAX_PLAN_STEPS {
        return Err(format!("Agent plans can contain at most {MAX_PLAN_STEPS} steps."));
    }

    for step in steps {
        validate_action_step(app, step)?;
    }
    Ok(())
}

fn validate_action_step(app: &AppHandle, step: &AgentStep) -> Result<(), String> {
    match step {
        AgentStep::LaunchApp { app: target } | AgentStep::SwitchToApp { app: target } => {
            resolve_app(target).map(|_| ())
        }
        AgentStep::RunRoutine { routine } => {
            let exists = find_routine_by_id(app, routine).is_some()
                || list_routines(app).ok().is_some_and(|items| {
                    items.iter().any(|item| {
                        item.name.eq_ignore_ascii_case(routine)
                            || item.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(routine))
                    })
                });
            if exists {
                Ok(())
            } else {
                Err(format!("Routine “{routine}” was not found."))
            }
        }
        AgentStep::DirectorPreset { preset } => {
            let exists = find_director_preset_by_id(app, preset).is_some()
                || load_director_presets(app).iter().any(|item| {
                    item.name.eq_ignore_ascii_case(preset)
                        || item.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(preset))
                });
            if exists {
                Ok(())
            } else {
                Err(format!("Director preset “{preset}” was not found."))
            }
        }
        AgentStep::Wait { milliseconds } => {
            if *milliseconds <= MAX_WAIT_MS {
                Ok(())
            } else {
                Err(format!(
                    "Agent waits cannot exceed {} seconds.",
                    MAX_WAIT_MS / 1000
                ))
            }
        }
        AgentStep::SavedAction { action } => {
            let saved = resolve_saved_action(app, action)
                .ok_or_else(|| format!("Saved AURA Action “{action}” was not found."))?;
            if matches!(saved.step, AgentStep::SavedAction { .. }) {
                return Err(
                    "Saved AURA Actions cannot recursively call another saved Action.".to_string(),
                );
            }
            Ok(())
        }
    }
}

fn validate_trigger(trigger: &AutomationTrigger) -> Result<(), String> {
    match trigger {
        AutomationTrigger::Startup => Ok(()),
        AutomationTrigger::Interval { every_minutes } => {
            if (1..=10_080).contains(every_minutes) {
                Ok(())
            } else {
                Err("Automation interval must be between 1 minute and 7 days.".to_string())
            }
        }
        AutomationTrigger::AtTime { run_at_ms, .. } => {
            if *run_at_ms > timestamp_ms() {
                Ok(())
            } else {
                Err("Scheduled automation time must be in the future.".to_string())
            }
        }
        AutomationTrigger::AppFocused { app } => resolve_app(app).map(|_| ()),
    }
}

fn initial_next_run(trigger: &AutomationTrigger, now: u64) -> Option<u64> {
    match trigger {
        AutomationTrigger::Interval { every_minutes } => {
            Some(now.saturating_add(every_minutes.saturating_mul(60_000)))
        }
        AutomationTrigger::AtTime { run_at_ms, .. } => Some(*run_at_ms),
        _ => None,
    }
}

fn app_matches_context(target: &str, process_name: &str) -> bool {
    resolve_app(target).is_ok_and(|target| {
        target
            .process_images()
            .iter()
            .any(|image| image.eq_ignore_ascii_case(process_name))
    })
}

fn resolve_app(value: &str) -> Result<AppTarget, String> {
    let normalized = normalize_phrase(value);
    AppTarget::from_alias(&normalized)
        .ok_or_else(|| format!("Unsupported application: {value}."))
}

fn resolve_saved_action(app: &AppHandle, value: &str) -> Option<SavedAuraAction> {
    let normalized = normalize_phrase(value);
    read_actions(app).ok()?.into_iter().find(|action| {
        action.id == value
            || normalize_phrase(&action.name) == normalized
            || action
                .aliases
                .iter()
                .any(|alias| normalize_phrase(alias) == normalized)
    })
}

fn parse_planned_payload(raw: &str) -> Result<PlannedPayload, String> {
    let trimmed = raw.trim();
    let without_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim();
    let cleaned = without_prefix
        .strip_suffix("```")
        .unwrap_or(without_prefix)
        .trim();

    if let Ok(payload) = serde_json::from_str::<PlannedPayload>(cleaned) {
        return Ok(payload);
    }

    let start = cleaned
        .find('{')
        .ok_or_else(|| "Agent planner did not return a JSON object.".to_string())?;
    let end = cleaned
        .rfind('}')
        .ok_or_else(|| "Agent planner returned incomplete JSON.".to_string())?;

    serde_json::from_str::<PlannedPayload>(&cleaned[start..=end])
        .map_err(|error| format!("Agent planner returned invalid JSON: {error}"))
}

fn update_run<F>(
    app: &AppHandle,
    runs: &Arc<Mutex<HashMap<String, AgentRun>>>,
    run_id: &str,
    update: F,
    message: &str,
) where
    F: FnOnce(&mut AgentRun),
{
    let snapshot = {
        let mut runs = runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(run) = runs.get_mut(run_id) else {
            return;
        };
        update(run);
        run.clone()
    };
    emit_agent_event(app, &snapshot, message);
}

fn finish_run(
    app: &AppHandle,
    runs: &Arc<Mutex<HashMap<String, AgentRun>>>,
    controls: &Arc<Mutex<HashMap<String, RunControl>>>,
    run_id: &str,
    state: &str,
    error: Option<String>,
) {
    let snapshot = {
        let mut runs = runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(run) = runs.get_mut(run_id) else {
            return;
        };
        run.state = state.to_string();
        run.current_step = None;
        run.completed_at_ms = Some(timestamp_ms());
        run.error = error;
        run.clone()
    };

    let _ = append_run_history(app, &snapshot);
    controls
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(run_id);
    emit_agent_event(
        app,
        &snapshot,
        match state {
            "completed" => "Agent completed the plan.",
            "cancelled" => "Agent run cancelled.",
            _ => "Agent run stopped.",
        },
    );
}

fn emit_agent_event(app: &AppHandle, run: &AgentRun, message: &str) {
    let _ = app.emit(
        AGENT_EVENT,
        AgentEvent {
            run: run.clone(),
            message: message.to_string(),
            timestamp_ms: timestamp_ms(),
        },
    );
}

fn append_run_history(app: &AppHandle, run: &AgentRun) -> Result<(), String> {
    let mut history = read_run_history(app).unwrap_or_default();
    history.retain(|item| item.id != run.id);
    history.insert(0, run.clone());
    history.truncate(MAX_HISTORY);
    write_json(app, RUN_HISTORY_FILENAME, &history)
}

fn read_run_history(app: &AppHandle) -> Result<Vec<AgentRun>, String> {
    read_json_or_default(app, RUN_HISTORY_FILENAME)
}

fn read_actions(app: &AppHandle) -> Result<Vec<SavedAuraAction>, String> {
    read_json_or_default(app, ACTIONS_FILENAME)
}

fn write_actions(app: &AppHandle, actions: &[SavedAuraAction]) -> Result<(), String> {
    write_json(app, ACTIONS_FILENAME, actions)
}

fn read_automations(app: &AppHandle) -> Result<Vec<AuraAutomation>, String> {
    read_json_or_default(app, AUTOMATIONS_FILENAME)
}

fn write_automations(app: &AppHandle, automations: &[AuraAutomation]) -> Result<(), String> {
    write_json(app, AUTOMATIONS_FILENAME, automations)
}

fn read_json_or_default<T>(app: &AppHandle, filename: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Default,
{
    let path = config_path(app, filename)?;
    if !path.exists() {
        return Ok(T::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {filename}: {error}"))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("{filename} is invalid and was left unchanged: {error}"))
}

fn write_json<T>(app: &AppHandle, filename: &str, value: &T) -> Result<(), String>
where
    T: Serialize + ?Sized,
{
    let path = config_path(app, filename)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn config_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(filename))
}

fn validate_text(value: &str, label: &str, max_chars: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label} cannot be empty."));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{label} is too long."));
    }
    Ok(value.to_string())
}

fn sanitize_aliases(
    aliases: &[String],
    name: &str,
    limit: usize,
) -> Result<Vec<String>, String> {
    let mut sanitized = Vec::<String>::new();
    for alias in aliases {
        let alias = validate_text(alias, "Alias", 80)?;
        if normalize_phrase(&alias) == normalize_phrase(name) {
            continue;
        }
        if !sanitized
            .iter()
            .any(|existing| normalize_phrase(existing) == normalize_phrase(&alias))
        {
            sanitized.push(alias);
        }
    }
    if sanitized.len() > limit {
        return Err(format!("At most {limit} aliases are supported."));
    }
    Ok(sanitized)
}

fn action_name_conflicts(
    existing: &SavedAuraAction,
    name: &str,
    aliases: &[String],
) -> bool {
    let mut requested = vec![normalize_phrase(name)];
    requested.extend(aliases.iter().map(|alias| normalize_phrase(alias)));

    let mut existing_names = vec![normalize_phrase(&existing.name)];
    existing_names.extend(existing.aliases.iter().map(|alias| normalize_phrase(alias)));

    requested.iter().any(|name| existing_names.contains(name))
}

fn step_label(step: &AgentStep) -> String {
    match step {
        AgentStep::LaunchApp { app } => format!("Launch {app}"),
        AgentStep::SwitchToApp { app } => format!("Switch to {app}"),
        AgentStep::RunRoutine { routine } => format!("Run routine {routine}"),
        AgentStep::DirectorPreset { preset } => format!("Director preset {preset}"),
        AgentStep::Wait { milliseconds } => format!("Wait {milliseconds} ms"),
        AgentStep::SavedAction { action } => format!("Run saved Action {action}"),
    }
}

fn normalize_phrase(value: &str) -> String {
    value
        .trim()
        .trim_matches(|character: char| matches!(character, '.' | ',' | '!' | '?' | ';' | ':'))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn slugify(value: &str) -> String {
    normalize_phrase(value)
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_inside_optional_markdown_fence() {
        let payload = parse_planned_payload(
            "```json\n{\"summary\":\"Open browser\",\"steps\":[{\"type\":\"launchApp\",\"app\":\"brave\"}]}\n```",
        )
        .unwrap();

        assert_eq!(payload.steps.len(), 1);
        assert_eq!(payload.summary, "Open browser");
    }

    #[test]
    fn permission_order_keeps_destructive_highest() {
        assert!(permission_rank(PermissionClass::Destructive)
            > permission_rank(PermissionClass::Sensitive));
        assert!(permission_rank(PermissionClass::Sensitive)
            > permission_rank(PermissionClass::Act));
    }

    #[test]
    fn interval_next_run_is_relative_to_now() {
        assert_eq!(
            initial_next_run(
                &AutomationTrigger::Interval { every_minutes: 5 },
                1_000,
            ),
            Some(301_000),
        );
    }

    #[test]
    fn action_alias_conflicts_are_case_insensitive() {
        let existing = SavedAuraAction {
            id: "action-1".to_string(),
            name: "Editing Mode".to_string(),
            description: String::new(),
            aliases: vec!["edit".to_string()],
            step: AgentStep::Wait { milliseconds: 1 },
            updated_at_ms: 1,
        };

        assert!(action_name_conflicts(
            &existing,
            "Other",
            &["EDIT".to_string()]
        ));
    }
}
