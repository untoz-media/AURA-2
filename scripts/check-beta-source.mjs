import { createHash } from "node:crypto";
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

requireFragments("apps/desktop/src-tauri/src/lib.rs", [
  "fn accept_wake_phrase(",
  "wake_transcript_is_playback_echo",
  "find_project_by_name_or_alias",
  "ActionIntent::ActivateProject",
]);

requireFragments("apps/desktop/src/bridge/aura.ts", [
  "update_memory_command",
  "start_obs_streaming",
  "confirmed",
]);

const validator = read("scripts/validate-beta.mjs");
if (!validator.includes('"/d", "/s", "/c"') || !validator.includes("ComSpec")) {
  failures.push("validate-beta.mjs must launch Windows .cmd shims through cmd.exe /d /s /c.");
}
if (/spawnSync\(\s*(?:npm\.cmd|["']npm\.cmd["'])/.test(validator)) {
  failures.push("validate-beta.mjs still spawns npm.cmd directly.");
}

const rootPackage = JSON.parse(read("package.json"));
for (const dependency of ["esbuild@0.28.2", "fsevents@2.3.3"]) {
  if (rootPackage.allowScripts?.[dependency] !== true) {
    failures.push(`package.json allowScripts must permit ${dependency}.`);
  }
}

const lock = JSON.parse(read("package-lock.json"));
if (lock.lockfileVersion < 3) {
  failures.push("package-lock.json must be an npm lockfileVersion 3 lock.");
}
if (!lock.packages || Object.keys(lock.packages).length < 10) {
  failures.push("package-lock.json does not look like a resolved npm lockfile.");
}

const reservedPowerShellVariables = new Set([
  "$",
  "_",
  "?",
  "^",
  "args",
  "consolefilename",
  "error",
  "event",
  "eventargs",
  "eventsubscriber",
  "executioncontext",
  "false",
  "foreach",
  "home",
  "host",
  "input",
  "matches",
  "myinvocation",
  "null",
  "pid",
  "profile",
  "psboundparameters",
  "pscmdlet",
  "pscommandpath",
  "psculture",
  "psdebugcontext",
  "pshome",
  "psitem",
  "psscriptroot",
  "pssenderinfo",
  "psuiculture",
  "psversiontable",
  "pwd",
  "sender",
  "shellid",
  "stacktrace",
  "switch",
  "this",
  "true",
]);

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function stripPowerShellComment(line) {
  let quote = null;
  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    if (quote) {
      if (char === "`") {
        index += 1;
        continue;
      }
      if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'") {
      quote = char;
      continue;
    }
    if (char === "#") return line.slice(0, index);
  }
  return line;
}

function reservedPowerShellName(name) {
  const bare = name.replace(/^(?:script|global|local|private|using):/i, "");
  return reservedPowerShellVariables.has(bare.toLowerCase()) ? bare : null;
}

function findReservedPowerShellAssignments(text) {
  const names = [...reservedPowerShellVariables].map(escapeRegExp).join("|");
  const scope = String.raw`(?:script|global|local|private|using):`;
  const assign = String.raw`(?:\+\+|--|[-+*/%]?=)`;
  const patterns = [
    new RegExp(String.raw`\$(?:${scope})?(${names})(?![A-Za-z0-9_])\s*${assign}`, "gi"),
    new RegExp(String.raw`\$\{(?:${scope})?(${names})\}(?![A-Za-z0-9_])\s*${assign}`, "gi"),
    new RegExp(String.raw`(?:\+\+|--)\$(?:${scope})?(${names})(?![A-Za-z0-9_])`, "gi"),
    new RegExp(String.raw`(?:\+\+|--)\$\{(?:${scope})?(${names})\}`, "gi"),
  ];
  const hits = [];

  text.split(/\r?\n/).forEach((rawLine, index) => {
    const line = stripPowerShellComment(rawLine);
    const found = new Set();
    for (const pattern of patterns) {
      pattern.lastIndex = 0;
      for (const match of line.matchAll(pattern)) {
        const reserved = reservedPowerShellName(match[1]);
        if (reserved) found.add(reserved.toLowerCase());
      }
    }
    const setter = line.match(/\b(?:Set-Variable|New-Variable)\b(.*)$/i);
    if (setter) {
      const named = setter[1].match(/-Name(?:\s+|:)['"]?([A-Za-z0-9_?^$]+)['"]?/i);
      const positional = setter[1].match(/^\s+['"]?([A-Za-z0-9_?^$]+)['"]?/);
      const candidate = named?.[1] ?? (positional && !positional[1].startsWith("-") ? positional[1] : null);
      const reserved = candidate ? reservedPowerShellName(candidate) : null;
      if (reserved) found.add(reserved.toLowerCase());
    }
    for (const name of found) {
      hits.push({ line: index + 1, name });
    }
  });

  return hits;
}

const reservedAssignmentSamples = [
  ["$host = 'x'", true],
  ["$Host += 1", true],
  ["$global:pid = 1", true],
  ["${pwd} = 'C:\\'", true],
  ["Set-Variable -Name Host -Value 1", true],
  ["New-Variable input -Value 1", true],
  ["$input++", true],
  ["++$args", true],
  ["$true = $false", true],
  ["$hostLine = 'x'", false],
  ["$ErrorActionPreference = 'Stop'", false],
  ["$matches[1]", false],
  ["$sdkLib = $null", false],
  ["$pythonOk = $false", false],
  ["$script:VsDevCmd = $null", false],
  ['Write-Host "$rustVersion - $rustHost"', false],
];
for (const [sample, shouldFlag] of reservedAssignmentSamples) {
  const flagged = findReservedPowerShellAssignments(sample).length > 0;
  if (flagged !== shouldFlag) {
    failures.push(`PowerShell reserved-variable detector mismatch for: ${sample}`);
  }
}

const ps1Files = fs
  .readdirSync("scripts")
  .filter((name) => name.endsWith(".ps1"))
  .map((name) => path.join("scripts", name));
for (const file of ps1Files) {
  const bytes = fs.readFileSync(file);
  if (bytes.some((byte) => byte > 127)) {
    failures.push(
      `${file} must stay ASCII so Windows PowerShell 5.1 does not misread a UTF-8 file without a BOM.`,
    );
  }
  const text = bytes.toString("ascii");
  if (text.includes("&&") || text.includes("||")) {
    failures.push(
      `${file} must not use && or ||. Windows PowerShell 5.1 rejects them as statement separators.`,
    );
  }
  for (const hit of findReservedPowerShellAssignments(text)) {
    failures.push(
      `${file}:${hit.line} assigns reserved PowerShell variable $${hit.name}. Rename it; Windows PowerShell 5.1 treats these as read-only or automatic.`,
    );
  }
}

for (const icon of [
  "apps/desktop/src-tauri/icons/icon.ico",
  "apps/desktop/src-tauri/icons/32x32.png",
  "apps/desktop/src-tauri/icons/128x128.png",
  "apps/desktop/src-tauri/icons/128x128@2x.png",
]) {
  if (!fs.existsSync(icon)) {
    failures.push(`${icon} must be committed so cargo test works on a fresh checkout.`);
  }
}
if (read(".gitignore").includes("apps/desktop/src-tauri/icons/")) {
  failures.push(".gitignore must not hide the committed Tauri icon set.");
}
if (!read(".gitignore").includes("apps/desktop/src-tauri/gen/")) {
  failures.push(".gitignore must ignore apps/desktop/src-tauri/gen/; Tauri rewrites it during a build.");
}
const iconStamp = "apps/desktop/src-tauri/icons/source.sha256";
const iconSvg = "apps/desktop/src-tauri/aura-app-icon.svg";
if (!fs.existsSync(iconStamp)) {
  failures.push(`${iconStamp} must record the SVG hash used for the committed icon set.`);
} else {
  const recorded = fs.readFileSync(iconStamp, "utf8").trim();
  const actual = createHash("sha256").update(fs.readFileSync(iconSvg)).digest("hex");
  if (recorded !== actual) {
    failures.push(`${iconStamp} does not match ${iconSvg}. Run npm run icons and commit the result.`);
  }
}
const desktopPackage = JSON.parse(read("apps/desktop/package.json"));
if (!desktopPackage.scripts?.["tauri:build"]?.includes("ensure-app-icons.mjs")) {
  failures.push("tauri:build must use ensure-app-icons.mjs so an up-to-date icon set is not regenerated.");
}

if (failures.length > 0) {
  console.error("AURA-2 Beta source checks failed:");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log(`AURA-2 Beta source checks OK: ${sourceFiles.length} source files scanned.`);
console.log("Permission, memory, OBS, skill, lockfile and PowerShell invariants are present.");
