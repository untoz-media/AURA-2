import fs from "node:fs";
import path from "node:path";

const roots = ["apps/desktop/src", "apps/desktop/src-tauri/src"];
const extensions = new Set([".ts", ".tsx", ".css", ".rs", ".py", ".json"]);

function walk(directory) {
  const entries = fs.readdirSync(directory, { withFileTypes: true });
  return entries.flatMap((entry) => {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) return walk(fullPath);
    return extensions.has(path.extname(entry.name)) ? [fullPath] : [];
  });
}

const files = roots.flatMap(walk);
const failures = [];

for (const file of files) {
  const content = fs.readFileSync(file, "utf8");
  if (/^(<<<<<<<|=======|>>>>>>>)(?: .*)?$/m.test(content)) {
    failures.push(`${file}: unresolved merge-conflict marker`);
  }
}

const staleCopy = [
  "Ready for M006.3 STT",
  "arrive later in M002",
  "Vision access will remain permission-based",
  "M005 · LOCAL CONTEXT",
  "M005.6 · PROJECT MEMORY",
  "Scheduling and autonomous background tasks remain scoped to",
];

for (const phrase of staleCopy) {
  for (const file of files) {
    if (fs.readFileSync(file, "utf8").includes(phrase)) {
      failures.push(`${file}: stale Alpha copy found: ${phrase}`);
    }
  }
}

function requireFragments(file, fragments) {
  const content = fs.readFileSync(file, "utf8");
  for (const fragment of fragments) {
    if (!content.includes(fragment)) {
      failures.push(`${file}: required Beta safety invariant missing: ${fragment}`);
    }
  }
}

requireFragments("apps/desktop/src-tauri/src/permissions.rs", [
  "PermissionClass::Sensitive | PermissionClass::Destructive",
  "PermissionDecision::Allow",
  "self.destructive = PermissionDecision::Ask",
  "self.sensitive = PermissionDecision::Ask",
  "pub fn fail_closed() -> Self",
]);

requireFragments("apps/desktop/src-tauri/src/lib.rs", [
  "return PermissionPolicy::fail_closed();",
  ".unwrap_or_else(|_| PermissionPolicy::fail_closed())",
]);

requireFragments("apps/desktop/src-tauri/src/lib.rs", [
  "paused: bool",
  "preferences.paused",
  ".set_paused(preferences.paused)",
  ".set_global_paused(app.handle(), preferences.paused)",
]);

requireFragments("apps/desktop/src-tauri/src/agents.rs", [
  "const MAX_PLAN_STEPS: usize = 12;",
  "const MAX_WAIT_MS: u64 = 30_000;",
  "PermissionClass::Read | PermissionClass::Act",
  "policy.decision_for(permission) == PermissionDecision::Allow",
  "AgentStep::LaunchApp { .. } | AgentStep::SwitchToApp { .. }",
  "Background automations cannot run sensitive routines.",
  "Background automations cannot run a sensitive Director preset.",
]);

const actionRouter = fs.readFileSync("apps/desktop/src-tauri/src/core/action_router.rs", "utf8");
const actionRouterRuntime = actionRouter.split("#[cfg(test)]")[0];
for (const forbidden of ["unreachable!(", ".expect("]) {
  if (actionRouterRuntime.includes(forbidden)) {
    failures.push(`apps/desktop/src-tauri/src/core/action_router.rs: runtime parser still relies on ${forbidden}`);
  }
}

const desktopLib = fs.readFileSync("apps/desktop/src-tauri/src/lib.rs", "utf8");
if (/fn finish_beta_self_test\([^)]*\)[^{]*\{\s*finish_beta_self_test\(/s.test(desktopLib)) {
  failures.push("apps/desktop/src-tauri/src/lib.rs: Beta self-test finisher is recursively calling itself");
}
if (!desktopLib.includes("finish_beta_self_test(checks)")) {
  failures.push("apps/desktop/src-tauri/src/lib.rs: runtime Beta self-test is not using the tested finalization helper");
}

const settingsSource = fs.readFileSync("apps/desktop/src/Settings.tsx", "utf8");
if (!settingsSource.includes('tone={!betaSelfTest.ready ? "critical"')) {
  failures.push("apps/desktop/src/Settings.tsx: non-ready Beta self-test must render as critical");
}

const atomicStores = [
  "apps/desktop/src-tauri/src/memory.rs",
  "apps/desktop/src-tauri/src/project_memory.rs",
  "apps/desktop/src-tauri/src/routines.rs",
  "apps/desktop/src-tauri/src/agents.rs",
  "apps/desktop/src-tauri/src/integrations/director.rs",
  "apps/desktop/src-tauri/src/model_manager.rs",
  "apps/desktop/src-tauri/src/vision_history.rs",
];
for (const file of atomicStores) {
  const source = fs.readFileSync(file, "utf8");
  if (!source.includes("write_json_atomic")) {
    failures.push(`${file}: critical local state is not using atomic JSON persistence`);
  }

  const directWritePattern =
    file.endsWith("model_manager.rs")
      ? /std_fs::write\s*\(/
      : /(?<!async_)fs::write\s*\(/;
  if (directWritePattern.test(source)) {
    failures.push(`${file}: critical local JSON store uses a direct file write instead of atomic persistence`);
  }
}

const desktopPersistence = fs.readFileSync("apps/desktop/src-tauri/src/lib.rs", "utf8");
for (const required of [
  "storage::write_json_atomic(&path, policy)",
  "storage::write_json_atomic(&path, preferences)",
]) {
  if (!desktopPersistence.includes(required)) {
    failures.push(`apps/desktop/src-tauri/src/lib.rs: desktop state persistence guard missing: ${required}`);
  }
}

const storageSource = fs.readFileSync("apps/desktop/src-tauri/src/storage.rs", "utf8");
for (const required of ["file.sync_all()", "fs::rename(&temporary, path)"]) {
  if (!storageSource.includes(required)) {
    failures.push(`apps/desktop/src-tauri/src/storage.rs: atomic write invariant missing: ${required}`);
  }
}

requireFragments("apps/desktop/src/Settings.tsx", [
  "export type SettingsSection =",
  '| "diagnostics";',
  "AURA-2 Beta Diagnostics",
  "Copy privacy-safe diagnostics",
  "Run self-test",
  "runBetaSelfTest",
  "Telemetry: automatic product telemetry off",
]);

requireFragments("apps/desktop/src-tauri/src/lib.rs", [
  "struct BetaSelfTestReport",
  "fn run_beta_self_test(",
  "run_beta_self_test,",
  "Permission policy passes Core sanitization.",
  "Saved Actions store loaded successfully",
  "Automations store loaded successfully",
]);

requireFragments("apps/desktop/src/bridge/aura.ts", [
  'invoke<BetaSelfTestReport>("run_beta_self_test")',
]);

requireFragments("apps/desktop/src/App.tsx", [
  "visionRuntime={visionRuntime}",
  "agentRuns={agentRuns}",
  "automations={automations}",
  "FIRST LOCAL SETUP",
  "const localSetupReady = managedRuntimeReady && localAssistantReady;",
  'openSettings("diagnostics")',
]);

const agents = fs.readFileSync("apps/desktop/src-tauri/src/agents.rs", "utf8");
if (/\#\[test\]\s*\n\s*\#\[test\]/.test(agents)) {
  failures.push("apps/desktop/src-tauri/src/agents.rs: duplicate #[test] attribute");
}

const requiredAgentTests = [
  "fn only_idempotent_app_steps_are_retried()",
  "fn interval_trigger_enforces_beta_bounds()",
  "fn planner_rejects_non_json_output()",
  "fn action_alias_conflicts_are_case_insensitive()",
];
for (const testName of requiredAgentTests) {
  const index = agents.indexOf(testName);
  if (index < 0) {
    failures.push(`apps/desktop/src-tauri/src/agents.rs: missing regression test ${testName}`);
    continue;
  }
  const prefix = agents.slice(Math.max(0, index - 80), index);
  if (!/\#\[test\]\s*$/.test(prefix.trimEnd())) {
    failures.push(`apps/desktop/src-tauri/src/agents.rs: ${testName} is not marked #[test]`);
  }
}

const settingsSource = fs.readFileSync("apps/desktop/src/Settings.tsx", "utf8");
const appSource = fs.readFileSync("apps/desktop/src/App.tsx", "utf8");
const propsStart = settingsSource.indexOf("type Props = {");
const propsEnd = settingsSource.indexOf("\n};", propsStart);
const settingsMountStart = appSource.indexOf("\n            <Settings\n");
const settingsMountEnd = appSource.indexOf("\n            />", settingsMountStart);

if (propsStart < 0 || propsEnd < 0 || settingsMountStart < 0 || settingsMountEnd < 0) {
  failures.push("Could not structurally validate Settings props wiring.");
} else {
  const propsBlock = settingsSource.slice(propsStart, propsEnd);
  const settingsMount = appSource.slice(settingsMountStart, settingsMountEnd);
  const requiredProps = [...propsBlock.matchAll(/^\s{2}([A-Za-z][A-Za-z0-9]*)(?:\??):/gm)]
    .map((match) => match[1]);
  const passedProps = [...settingsMount.matchAll(/^\s+([A-Za-z][A-Za-z0-9]*)=/gm)]
    .map((match) => match[1]);

  for (const prop of requiredProps) {
    if (!passedProps.includes(prop)) {
      failures.push(`apps/desktop/src/App.tsx: Settings prop is not wired: ${prop}`);
    }
  }

  for (const prop of passedProps) {
    if (!requiredProps.includes(prop)) {
      failures.push(`apps/desktop/src/App.tsx: unknown Settings prop is wired: ${prop}`);
    }
  }
}

if (failures.length > 0) {
  console.error("AURA-2 Beta source checks failed:");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log(`AURA-2 Beta source checks OK: ${files.length} source files scanned.`);
console.log("Permission and Agent safety invariants are present.");
