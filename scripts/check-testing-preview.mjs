import fs from "node:fs";

const fail = (message) => {
  console.error(`FAIL  ${message}`);
  process.exitCode = 1;
};

const pass = (message) => console.log(`PASS  ${message}`);

const root = JSON.parse(fs.readFileSync("package.json", "utf8"));
const desktop = JSON.parse(fs.readFileSync("apps/desktop/package.json", "utf8"));
const tauri = JSON.parse(fs.readFileSync("apps/desktop/src-tauri/tauri.conf.json", "utf8"));
const cargo = fs.readFileSync("apps/desktop/src-tauri/Cargo.toml", "utf8");
const beta = fs.readFileSync("apps/desktop/src-tauri/src/beta.rs", "utf8");
const lib = fs.readFileSync("apps/desktop/src-tauri/src/lib.rs", "utf8");
const types = fs.readFileSync("apps/desktop/src/bridge/types.ts", "utf8");
const settings = fs.readFileSync("apps/desktop/src/Settings.tsx", "utf8");
const bridge = fs.readFileSync("apps/desktop/src/bridge/aura.ts", "utf8");
const app = fs.readFileSync("apps/desktop/src/App.tsx", "utf8");
const releaseWorkflow = fs.readFileSync(".github/workflows/testing-preview-release.yml", "utf8");

const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const checks = [
  ["root version is a 0.9.0 Beta candidate", /^0\.9\.0-beta\.\d+$/.test(root.version)],
  ["desktop version matches root", desktop.version === root.version],
  ["Tauri version matches root", tauri.version === root.version],
  ["Cargo version matches root", cargoVersion === root.version],
  ["Windows NSIS bundling is enabled", tauri.bundle?.active === true && tauri.bundle?.targets?.includes("nsis")],
  ["installer is current-user only", tauri.bundle?.windows?.nsis?.installMode === "currentUser"],
  ["downgrades are blocked", tauri.bundle?.windows?.allowDowngrades === false],
  ["testing preview notes exist", fs.existsSync("docs/ALPHA-V1-TESTING-PREVIEW.md")],
  ["security policy exists", fs.existsSync("SECURITY.md")],
  ["bug report form exists", fs.existsSync(".github/ISSUE_TEMPLATE/bug_report.yml")],
  ["Windows Build Doctor exists", fs.existsSync("scripts/windows-build-doctor.ps1")],
  ["Windows Build Doctor is wired to npm", root.scripts?.["beta:doctor:windows"]?.includes("windows-build-doctor.ps1")],
  ["local Beta build uses Build Doctor", fs.readFileSync("scripts/build-beta-windows.ps1", "utf8").includes("windows-build-doctor.ps1")],
  ["build provenance schema is v3", beta.includes("build_commit") && beta.includes("build_source") && beta.includes("build_label")],
  ["backend injects compile-time provenance", lib.includes('option_env!("AURA_BUILD_COMMIT")') && lib.includes('option_env!("AURA_BUILD_SOURCE")')],
  ["desktop types expose provenance", types.includes("buildCommit: string") && types.includes("buildSource: string") && types.includes("buildLabel: string")],
  ["Diagnostics UI exposes provenance", settings.includes('title="Build provenance"')],
  ["crash-loop recovery state is persisted", beta.includes("unclean_streak: u32") && beta.includes("CRASH_LOOP_THRESHOLD")],
  ["crash-loop guard suppresses background startup", lib.includes("&& !crash_loop_guard_active") && lib.includes("beta.crash_loop_guard")],
  ["desktop types expose crash-loop state", types.includes("consecutiveUncleanSessions: number") && types.includes("crashLoopGuardActive: boolean")],
  ["Testing Preview readiness UI exists", settings.includes("Testing Preview readiness") && settings.includes("betaBuildTraceable") && settings.includes("betaPlatformCompatible")],
  ["Recovery resume is fail-closed", lib.includes("recovery_override") && lib.includes("Review Beta & Diagnostics")],
  ["Recovery override is explicit in desktop UI", settings.includes("Review & Resume") && settings.includes("onPausedChange(false, true)")],
  ["Testing Preview builds identify themselves", app.includes("AURA-2 TESTING PREVIEW") && app.includes("beta-local-smoke") && app.includes("buildCommit.slice(0, 12)")],
  ["Beta Test Session backend is fixed-area and privacy-safe", beta.includes("BETA_TEST_AREAS") && beta.includes("TEST_REPORT_MAX_BYTES") && beta.includes("normalize_test_session")],
  ["Beta Test Session commands are bridged", bridge.includes("get_beta_test_session") && bridge.includes("export_beta_test_report")],
  ["Beta Test Session UI exists", settings.includes("Beta Test Session") && settings.includes("Export report") && settings.includes("free-text notes")],
  ["preview release is a prerelease", releaseWorkflow.includes("prerelease: true")],
  ["preview release is not latest", releaseWorkflow.includes("make_latest: false")],
  ["preview workflow attaches checksum and manifest", releaseWorkflow.includes("AURA-2-Windows-x64.sha256") && releaseWorkflow.includes("AURA-2-Testing-Preview-Build.json")],
  ["preview workflow captures both dependency locks", releaseWorkflow.includes("package-lock.json") && releaseWorkflow.includes("apps/desktop/src-tauri/Cargo.lock") && releaseWorkflow.includes("cargoLockSha256")],
];

console.log(`AURA-2 testing preview structural gate — ${root.version}`);
for (const [label, ok] of checks) {
  ok ? pass(label) : fail(label);
}

if (process.exitCode) {
  console.error("\nTesting preview structural gate failed.");
  process.exit(process.exitCode);
}

console.log("\nTesting preview structural gate passed.");
