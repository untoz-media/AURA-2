import fs from "node:fs";
import path from "node:path";

const ROOT = path.join("apps", "desktop", "src-tauri", "src");
const FORBIDDEN = [
  { label: "unwrap()", pattern: /\.\s*unwrap\s*\(/g },
  { label: "expect()", pattern: /\.\s*expect\s*\(/g },
  { label: "unreachable!()", pattern: /\bunreachable!\s*\(/g },
  { label: "panic!()", pattern: /\bpanic!\s*\(/g },
  { label: "todo!()", pattern: /\btodo!\s*\(/g },
  { label: "unimplemented!()", pattern: /\bunimplemented!\s*\(/g },
];

function walk(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) return walk(full);
    return entry.isFile() && entry.name.endsWith(".rs") ? [full] : [];
  });
}

function productionPrefix(source) {
  const testMarker = source.indexOf("#[cfg(test)]");
  return testMarker >= 0 ? source.slice(0, testMarker) : source;
}

function stripLineComment(line) {
  const index = line.indexOf("//");
  return index >= 0 ? line.slice(0, index) : line;
}

const files = walk(ROOT).filter(
  (file) => !file.endsWith(`${path.sep}validation_tests.rs`),
);
const failures = [];

for (const file of files) {
  const source = productionPrefix(fs.readFileSync(file, "utf8"));
  const sourceLines = source.split(/\r?\n/);

  for (let index = 0; index < sourceLines.length; index += 1) {
    const line = stripLineComment(sourceLines[index]);

    for (const rule of FORBIDDEN) {
      rule.pattern.lastIndex = 0;
      if (rule.pattern.test(line)) {
        failures.push({
          file: file.replaceAll("\\", "/"),
          line: index + 1,
          rule: rule.label,
          source: sourceLines[index].trim(),
        });
      }
    }
  }
}

console.log("AURA-2 production Rust panic-surface gate");
console.log(`Scanned ${files.length} Rust source files.`);

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(
      `FAIL  ${failure.file}:${failure.line} — ${failure.rule} — ${failure.source}`,
    );
  }
  console.error(
    `\nFound ${failures.length} panic-prone production Rust construct(s). Use Result/Option handling or another fail-closed path instead.`,
  );
  process.exit(1);
}

console.log(
  "PASS  No unwrap/expect/unreachable/panic/todo/unimplemented constructs were found in the scanned production Rust surface.",
);
