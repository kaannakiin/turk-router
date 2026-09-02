/**
 * The TypeScript half of the eight-lint discipline is policy: the tsconfig flags, the oxlint
 * rules, the import allow-list, and the rule that wire literals live in one module. Nothing else
 * fails a change that drops or narrows one of them, so this does. `tsconfig.json` is read as text
 * because it carries comments; each pinned flag sits on its own line.
 */
import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { arrayField, field, isRecord, packageRoot, readText, stringField } from "./common/paths.js";

const TSCONFIG_FLAGS = [
  '"strict": true',
  '"exactOptionalPropertyTypes": true',
  '"noUncheckedIndexedAccess": true',
  '"noPropertyAccessFromIndexSignature": true',
  '"noImplicitOverride": true',
  '"noImplicitReturns": true',
  '"noFallthroughCasesInSwitch": true',
  '"noUnusedLocals": true',
  '"noUnusedParameters": true',
  '"allowUnreachableCode": false',
  '"allowUnusedLabels": false',
  '"verbatimModuleSyntax": true',
  '"erasableSyntaxOnly": true',
  '"isolatedDeclarations": true',
  '"module": "nodenext"',
  '"moduleResolution": "nodenext"',
];

const OXLINT_RULES = [
  "typescript/no-explicit-any",
  "typescript/no-non-null-assertion",
  "typescript/no-unsafe-type-assertion",
  "typescript/no-unsafe-assignment",
  "typescript/no-unsafe-member-access",
  "typescript/no-unsafe-call",
  "typescript/no-unsafe-return",
  "typescript/no-unsafe-argument",
  "typescript/switch-exhaustiveness-check",
  "typescript/no-floating-promises",
  "typescript/restrict-template-expressions",
  "typescript/strict-boolean-expressions",
  "typescript/no-unnecessary-condition",
  "typescript/ban-ts-comment",
  "typescript/array-type",
  "typescript/consistent-type-imports",
];

const SRC_IMPORTS = [
  "@solana/addresses",
  "@solana/codecs-core",
  "@solana/codecs-data-structures",
  "@solana/codecs-numbers",
  "@solana/instructions",
];

const BANNED_IN_SRC = [
  ": any",
  "as any",
  "<any>",
  "Array<any>",
  "@ts-ignore",
  "@ts-expect-error",
  "@ts-nocheck",
  "as unknown as",
  "eslint-disable",
  "oxlint-disable",
];

function sources(directory: string): Array<[string, string]> {
  return readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".ts"))
    .map((entry) => join(entry.parentPath, entry.name))
    .sort()
    .map((path) => [path, readText(path)]);
}

function specifiers(text: string): Array<string> {
  return [...text.matchAll(/from "([^"]+)"/g)].map((match) => match[1] ?? "");
}

test("tsconfig pins the discipline", () => {
  const lines = readText(join(packageRoot, "tsconfig.json")).split("\n");
  for (const flag of TSCONFIG_FLAGS) {
    assert.ok(
      lines.some((line) => line.trim().startsWith(flag)),
      `${flag} is missing from tsconfig.json`,
    );
  }
});

test("the lint config carries every rule at error", () => {
  const config: unknown = JSON.parse(readText(join(packageRoot, ".oxlintrc.json")));
  assert.equal(field(field(config, "options"), "typeAware"), true);
  const rules = field(config, "rules");
  assert.ok(isRecord(rules));
  for (const rule of OXLINT_RULES) {
    const level: unknown = rules[rule];
    const severity: unknown = Array.isArray(level) ? level[0] : level;
    assert.equal(severity, "error", rule);
  }
});

test("no source file weakens the checker", () => {
  for (const [path, text] of sources(join(packageRoot, "src"))) {
    for (const token of BANNED_IN_SRC) {
      assert.ok(!text.includes(token), `${path} contains ${token}`);
    }
    assert.ok(!/\w\[\]/.test(text), `${path} uses array sugar; write Array<T>`);
  }
});

test("src imports only the granular Kit packages", () => {
  for (const [path, text] of sources(join(packageRoot, "src"))) {
    for (const specifier of specifiers(text)) {
      if (specifier.startsWith(".")) {
        continue;
      }
      assert.ok(SRC_IMPORTS.includes(specifier), `${path} imports ${specifier}`);
    }
  }
  for (const [path, text] of sources(join(packageRoot, "tests"))) {
    for (const specifier of specifiers(text)) {
      assert.ok(
        !["@solana/kit", "@solana/web3.js", "bs58", "@solana-program/token"].includes(specifier),
        `${path} imports ${specifier}`,
      );
    }
  }
});

test("the package is private ESM with pinned dependencies", () => {
  const manifest: unknown = JSON.parse(readText(join(packageRoot, "package.json")));
  assert.equal(field(manifest, "type"), "module");
  assert.equal(field(manifest, "private"), true);
  assert.ok(stringField(field(manifest, "engines"), "node").startsWith(">=22"));
  const peers = field(manifest, "peerDependencies");
  assert.ok(isRecord(peers));
  assert.deepEqual(
    Object.keys(peers).sort((a, b) => a.localeCompare(b)),
    SRC_IMPORTS,
  );
  for (const range of Object.values(peers)) {
    assert.ok(typeof range === "string" && range.startsWith("^8."), String(range));
  }
  const dev = field(manifest, "devDependencies");
  assert.ok(isRecord(dev));
  for (const [name, version] of Object.entries(dev)) {
    assert.ok(
      typeof version === "string" && /^\d+\.\d+\.\d+$/.test(version),
      `${name}: ${String(version)}`,
    );
  }
  assert.deepEqual(arrayField(manifest, "files"), ["dist", "README.md"]);
});

test("wire literals live only in src/wire.ts", () => {
  for (const [path, text] of sources(join(packageRoot, "src"))) {
    if (path.endsWith("wire.ts")) {
      continue;
    }
    for (const literal of ["TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am", "0x63, 0x61, 0x70"]) {
      assert.ok(!text.includes(literal), `${path} carries a wire literal`);
    }
  }
});
