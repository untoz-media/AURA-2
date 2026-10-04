import fs from "node:fs";

const rootPackage = JSON.parse(fs.readFileSync("package.json", "utf8"));
const desktopPackage = JSON.parse(fs.readFileSync("apps/desktop/package.json", "utf8"));
const tauriConfig = JSON.parse(fs.readFileSync("apps/desktop/src-tauri/tauri.conf.json", "utf8"));
const cargoToml = fs.readFileSync("apps/desktop/src-tauri/Cargo.toml", "utf8");

const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const versions = {
  rootPackage: rootPackage.version,
  desktopPackage: desktopPackage.version,
  tauri: tauriConfig.version,
  cargo: cargoVersion,
};

const missingVersion = Object.entries(versions).find(([, value]) => !value);
if (missingVersion) {
  throw new Error(`Missing release version in ${missingVersion[0]}.`);
}

const distinctVersions = new Set(Object.values(versions));
if (distinctVersions.size !== 1) {
  console.error("AURA-2 release versions are not synchronized:");
  for (const [source, version] of Object.entries(versions)) {
    console.error(`  ${source}: ${version}`);
  }
  process.exit(1);
}

const bundle = tauriConfig.bundle;
if (!bundle?.active || !bundle.targets?.includes("nsis")) {
  throw new Error("Windows release packaging must keep the NSIS target enabled.");
}

if (bundle.windows?.nsis?.installMode !== "currentUser") {
  throw new Error("Beta installer must use the currentUser NSIS install mode.");
}

if (bundle.windows?.allowDowngrades !== false) {
  throw new Error("Beta installer must reject accidental downgrades.");
}

if (bundle.windows?.webviewInstallMode?.type !== "embedBootstrapper") {
  throw new Error("Beta installer must embed the WebView2 bootstrapper.");
}

console.log(`AURA-2 release metadata OK: ${rootPackage.version}`);
console.log("NSIS installer policy OK: current-user, no downgrade, embedded WebView2 bootstrapper.");
