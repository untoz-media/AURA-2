import fs from "node:fs";

const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
const roadmap = fs.readFileSync("ROADMAP.md", "utf8");
const releaseNotes = fs.readFileSync(`docs/RELEASE-${pkg.version}.md`, "utf8");
const blockers = [];

if (!fs.existsSync("package-lock.json")) {
  blockers.push("package-lock.json is missing.");
}

if (!releaseNotes.includes(pkg.version) || !releaseNotes.includes("Public Beta")) {
  blockers.push("Release notes do not identify this version as a Public Beta candidate.");
}

const m0093 = roadmap.match(/- \[[ x]\] \*\*M009\.3\*\*/);
const m0096 = roadmap.match(/- \[[ x]\] \*\*M009\.6\*\*/);
if (!m0093 || m0093[0].includes("[x]")) {
  blockers.push("M009.3 must stay open until a real Windows installer is smoke-tested.");
}
if (!m0096 || m0096[0].includes("[x]")) {
  blockers.push("M009.6 must stay open until the Public Beta is actually published.");
}

blockers.push(
  "GitHub-hosted Actions are account-locked before step 1 (billing). Representative annotation: Windows Build job 111957243513, run 37367842805: \"The job was not started because your account is locked due to a billing issue.\"",
);
blockers.push(
  "This environment cannot build or verify the Windows NSIS installer. Run npm run beta:build:windows and npm run beta:verify:artifact on a Windows machine.",
);

console.error("AURA-2 Public Beta publication is BLOCKED:");
for (const blocker of blockers) console.error(`  - ${blocker}`);
console.error("Local source gates can still pass. Do not tag or publish 0.9.0-beta.1.");
process.exit(2);
