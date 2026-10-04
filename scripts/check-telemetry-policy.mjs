import fs from "node:fs";

const policyPath = "docs/TELEMETRY-POLICY.md";
const failures = [];

if (!fs.existsSync(policyPath)) {
  failures.push(`${policyPath}: policy document is missing`);
} else {
  const policy = fs.readFileSync(policyPath, "utf8");
  for (const required of ["**Default:** Off", "sends no automatic product telemetry to Untoz"]) {
    if (!policy.includes(required)) failures.push(`${policyPath}: missing required statement: ${required}`);
  }
}

const manifests = [
  ["package.json", fs.readFileSync("package.json", "utf8")],
  ["apps/desktop/package.json", fs.readFileSync("apps/desktop/package.json", "utf8")],
  ["apps/desktop/src-tauri/Cargo.toml", fs.readFileSync("apps/desktop/src-tauri/Cargo.toml", "utf8")],
];

const blockedSdkTokens = [
  "sentry",
  "posthog",
  "mixpanel",
  "amplitude",
  "segment-analytics",
  "@segment/analytics",
  "datadog",
];

for (const [file, content] of manifests) {
  const normalized = content.toLowerCase();
  for (const token of blockedSdkTokens) {
    if (normalized.includes(token)) {
      failures.push(`${file}: telemetry SDK token present without a policy change: ${token}`);
    }
  }
}

if (failures.length) {
  console.error("AURA-2 telemetry policy checks failed:");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log("AURA-2 telemetry policy OK: automatic product telemetry remains Off.");
