import { spawnSync } from "node:child_process";

const isWindows = process.platform === "win32";
const npm = isWindows ? "npm.cmd" : "npm";
const cargo = isWindows ? "cargo.exe" : "cargo";

function run(label, command, args) {
  console.log(`\n==> ${label}`);
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: false,
  });

  if (result.error) {
    console.error(`Could not start ${command}: ${result.error.message}`);
    process.exit(1);
  }

  if (result.status !== 0) {
    console.error(`${label} failed with exit code ${result.status ?? "unknown"}.`);
    process.exit(result.status ?? 1);
  }
}

function runPythonCompile() {
  const files = [
    "apps/desktop/src-tauri/src/model_runtime.py",
    "apps/desktop/src-tauri/src/speech_runtime.py",
    "apps/desktop/src-tauri/src/tts_runtime.py",
    "apps/desktop/src-tauri/src/vision_runtime.py",
  ];

  const candidates = process.env.PYTHON
    ? [[process.env.PYTHON, []]]
    : isWindows
      ? [["python", []], ["py", ["-3.12"]]]
      : [["python3", []], ["python", []]];

  for (const [command, prefix] of candidates) {
    const probe = spawnSync(command, [...prefix, "--version"], {
      stdio: "ignore",
      shell: false,
    });
    if (!probe.error && probe.status === 0) {
      run("Python worker syntax", command, [...prefix, "-m", "py_compile", ...files]);
      return;
    }
  }

  console.error("Python 3 was not found. Install Python 3.12 or set the PYTHON environment variable.");
  process.exit(1);
}

console.log("AURA-2 Public Beta source validation");
console.log(`Platform: ${process.platform} ${process.arch}`);

run("Version metadata", process.execPath, ["scripts/verify-versions.mjs"]);
run("Desktop TypeScript + Vite build", npm, [
  "--workspace",
  "@untoz/aura-desktop",
  "run",
  "build",
]);
runPythonCompile();
run("Rust formatting", cargo, [
  "fmt",
  "--manifest-path",
  "apps/desktop/src-tauri/Cargo.toml",
  "--",
  "--check",
]);

console.log("\nAURA-2 Beta source validation passed.");
