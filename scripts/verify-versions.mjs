import fs from "node:fs";

const root = JSON.parse(fs.readFileSync("package.json", "utf8"));
const desktop = JSON.parse(
  fs.readFileSync("apps/desktop/package.json", "utf8"),
);
const tauri = JSON.parse(
  fs.readFileSync("apps/desktop/src-tauri/tauri.conf.json", "utf8"),
);
const cargo = fs.readFileSync(
  "apps/desktop/src-tauri/Cargo.toml",
  "utf8",
);

const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const versions = {
  root: root.version,
  desktop: desktop.version,
  tauri: tauri.version,
  cargo: cargoVersion,
};

const unique = new Set(Object.values(versions));
if (unique.size !== 1 || [...unique].some((version) => !version)) {
  console.error("AURA version metadata is inconsistent:", versions);
  process.exit(1);
}

console.log(`AURA version metadata is consistent: ${root.version}`);
