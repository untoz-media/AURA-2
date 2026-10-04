import fs from "node:fs";

const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
const version = pkg.version;
const failures = [];

if (!/^\d+\.\d+\.\d+-beta\.\d+$/.test(version)) {
  failures.push(`Root version is not a Beta semver: ${version}`);
}

if (!fs.existsSync("package-lock.json")) {
  failures.push("package-lock.json is required for a reproducible Public Beta build.");
}

const requiredDocs = [
  "docs/BETA.md",
  "docs/BETA-GUIDE.md",
  "docs/BETA-TEST-CHECKLIST.md",
  "docs/BETA-KNOWN-ISSUES.md",
  "docs/TELEMETRY-POLICY.md",
  `RELEASE-${version}.md`,
];

for (const file of requiredDocs) {
  if (!fs.existsSync(file)) failures.push(`Missing Beta release document: ${file}`);
}

const roadmap = fs.readFileSync("ROADMAP.md", "utf8");
for (const milestone of ["M009.1", "M009.2", "M009.3", "M009.4", "M009.5"]) {
  if (!roadmap.includes(`- [x] **${milestone}**`)) {
    failures.push(`${milestone} must be complete before tagging the Public Beta.`);
  }
}

const telemetry = fs.readFileSync("docs/TELEMETRY-POLICY.md", "utf8");
if (!telemetry.includes("**Default:** Off")) {
  failures.push("Telemetry policy must remain Off for this Beta release.");
}

const releaseNotesPath = `RELEASE-${version}.md`;
if (fs.existsSync(releaseNotesPath)) {
  const releaseNotes = fs.readFileSync(releaseNotesPath, "utf8");
  if (!releaseNotes.includes(version)) failures.push(`${releaseNotesPath}: version is missing from release notes.`);
  if (!releaseNotes.includes("Public Beta")) failures.push(`${releaseNotesPath}: Public Beta label is missing.`);
}

if (failures.length) {
  console.error("AURA-2 Public Beta release gate is BLOCKED:");
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}

console.log(`AURA-2 ${version} Public Beta release metadata is ready.`);
console.log("All pre-release milestones M009.1–M009.5 are marked complete.");
