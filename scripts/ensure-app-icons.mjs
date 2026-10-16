import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const desktop = path.join(root, "apps", "desktop");
const svgPath = path.join(desktop, "src-tauri", "aura-app-icon.svg");
const iconsDir = path.join(desktop, "src-tauri", "icons");
const stampPath = path.join(iconsDir, "source.sha256");
const required = [
  "icon.ico",
  "icon.icns",
  "32x32.png",
  "128x128.png",
  "128x128@2x.png",
  "android/mipmap-anydpi-v26/ic_launcher.xml",
  "android/values/ic_launcher_background.xml",
];

const svgHash = createHash("sha256").update(fs.readFileSync(svgPath)).digest("hex");
const stamp = fs.existsSync(stampPath) ? fs.readFileSync(stampPath, "utf8").trim() : "";
const present = required.every((file) => fs.existsSync(path.join(iconsDir, file)));
const force = process.argv.includes("--force");

if (!force && stamp === svgHash && present) {
  console.log("AURA app icons match aura-app-icon.svg; skipping regeneration.");
  process.exit(0);
}

const cli = path.join(root, "node_modules", "@tauri-apps", "cli", "tauri.js");
const result = spawnSync(process.execPath, [cli, "icon", "src-tauri/aura-app-icon.svg"], {
  cwd: desktop,
  stdio: "inherit",
});
if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

fs.writeFileSync(stampPath, `${svgHash}\n`);
console.log("AURA app icons regenerated from aura-app-icon.svg.");
