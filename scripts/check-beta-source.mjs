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

requireFragments("apps/desktop/src/Settings.tsx", [
  "export type SettingsSection =",
  '| "diagnostics";',
  "AURA-2 Beta Diagnostics",
  "Copy privacy-safe diagnostics",
  "Telemetry: automatic product telemetry off",
]);

requireFragments("apps/desktop/src/App.tsx", [
  "visionRuntime={visionRuntime}",
  "agentRuns={agentRuns}",
  "automations={automations}",
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

if (failures.length > 0) {
  console.error("AURA-2 Beta source checks failed:");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log(`AURA-2 Beta source checks OK: ${files.length} source files scanned.`);
console.log("Permission and Agent safety invariants are present.");
