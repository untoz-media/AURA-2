import fs from "node:fs";
import path from "node:path";

const failures = [];

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function requireFragments(file, fragments) {
  const content = read(file);
  for (const fragment of fragments) {
    if (!content.includes(fragment)) {
      failures.push(`${file}: missing required fragment: ${fragment}`);
    }
  }
}

function walk(directory, extensions) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === "target") return [];
      return walk(fullPath, extensions);
    }
    return extensions.has(path.extname(entry.name)) ? [fullPath] : [];
  });
}

const sourceFiles = [
  ...walk("apps/desktop/src", new Set([".ts", ".tsx", ".css"])),
  ...walk("apps/desktop/src-tauri/src", new Set([".rs", ".py"])),
];

for (const file of sourceFiles) {
  const content = read(file);
  if (/^(<<<<<<<|=======|>>>>>>>)(?: .*)?$/m.test(content)) {
    failures.push(`${file}: unresolved merge-conflict marker`);
  }
}

const staleUi = [
  "M005 · LOCAL CONTEXT",
  "M005.6 · PROJECT MEMORY",
  "Automatic context capture is not enabled",
  "Ready for M006.3 STT",
  "Desktop Foundation / 0.2.0",
];

for (const phrase of staleUi) {
  for (const file of sourceFiles.filter((item) => item.endsWith(".tsx") || item.endsWith(".ts"))) {
    if (read(file).includes(phrase)) {
      failures.push(`${file}: stale user-facing copy: ${phrase}`);
    }
  }
}

requireFragments("apps/desktop/src-tauri/src/lib.rs", [
  "fn authorize_explicit_desktop_action(",
  "PermissionClass::Sensitive",
  "Starting an OBS stream",
  "preset_requires_sensitive_permission",
  "update_memory_command",
]);

requireFragments("apps/desktop/src-tauri/src/beta.rs", [
  "telemetry_enabled: false",
  "automatic_crash_uploads: false",
]);

requireFragments("apps/desktop/src-tauri/src/permissions.rs", [
  "PermissionClass::Sensitive | PermissionClass::Destructive",
  "Sensitive and destructive permissions cannot be permanently allowed.",
]);

requireFragments("apps/desktop/src-tauri/src/computer/app_skills.rs", [
  "execute_focus_verified_shortcut",
  "explorer.desktop",
  "\"browser.{}.{}\"",
  "\"notepad.{}\"",
  "\"terminal.{}\"",
  "\"calculator.{}\"",
  "\"new-tab\"",
  "\"new-note\"",
  "\"standard-mode\"",
]);

const skills = read("apps/desktop/src-tauri/src/computer/app_skills.rs");
for (const forbidden of ["powershell", "cmd.exe", "sh -c", "wt.exe "]) {
  if (skills.toLowerCase().includes(forbidden)) {
    failures.push(`app_skills.rs contains a shell-style launcher: ${forbidden}`);
  }
}

requireFragments("apps/desktop/src-tauri/capabilities/default.json", [
  "core:default",
]);
const capabilities = read("apps/desktop/src-tauri/capabilities/default.json");
if (capabilities.includes("shell:allow") || capabilities.includes("fs:scope")) {
  failures.push("Tauri capabilities grant a shell or broad filesystem scope.");
}

requireFragments("apps/desktop/src/Create.tsx", [
  "Video generation is next, not simulated.",
  "Planned",
]);

requireFragments("apps/desktop/src/Memory.tsx", [
  "onUpdate",
  "foreground app",
  "does not include file",
]);

requireFragments("apps/desktop/src/bridge/aura.ts", [
  "update_memory_command",
  "start_obs_streaming",
  "confirmed",
]);

const lock = JSON.parse(read("package-lock.json"));
if (lock.lockfileVersion < 3) {
  failures.push("package-lock.json must be an npm lockfileVersion 3 lock.");
}
if (!lock.packages || Object.keys(lock.packages).length < 10) {
  failures.push("package-lock.json does not look like a resolved npm lockfile.");
}

if (failures.length > 0) {
  console.error("AURA-2 Beta source checks failed:");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log(`AURA-2 Beta source checks OK: ${sourceFiles.length} source files scanned.`);
console.log("Permission, memory, OBS, skill and lockfile invariants are present.");
