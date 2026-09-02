import { readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));

export const repoRoot: string = resolve(here, "../../../..");
export const wireRoot: string = join(repoRoot, "wire");
export const goldenPath: string = join(repoRoot, "clients", "golden", "find_route.json");
export const packageRoot: string = resolve(here, "../..");

export function readText(path: string): string {
  return readFileSync(path, "utf8");
}

export function manifest(): unknown {
  return JSON.parse(readText(join(wireRoot, "wire-manifest.json")));
}

/**
 * Every fixture under `wire/fixtures`, sorted by path. Nothing may assume a corpus name, a count
 * or a synthetic address: the delivery that carries a new wire replaces the tree wholesale.
 */
export function fixturePaths(): Array<string> {
  const root = join(wireRoot, "fixtures");
  const found = readdirSync(root, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name === "accounts.txt")
    .map((entry) => join(entry.parentPath, entry.name))
    .sort();
  if (found.length === 0) {
    throw new Error("wire/fixtures holds no fixture");
  }
  return found;
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function field(value: unknown, key: string): unknown {
  if (!isRecord(value) || !(key in value)) {
    throw new Error(`no field ${key}`);
  }
  return value[key];
}

export function numberField(value: unknown, key: string): number {
  const found = field(value, key);
  if (typeof found !== "number") {
    throw new Error(`${key} is not a number`);
  }
  return found;
}

export function stringField(value: unknown, key: string): string {
  const found = field(value, key);
  if (typeof found !== "string") {
    throw new Error(`${key} is not a string`);
  }
  return found;
}

export function booleanField(value: unknown, key: string): boolean {
  const found = field(value, key);
  if (typeof found !== "boolean") {
    throw new Error(`${key} is not a boolean`);
  }
  return found;
}

export function asArray(value: unknown): Array<unknown> {
  if (!Array.isArray(value)) {
    throw new Error("not an array");
  }
  return value;
}

export function arrayField(value: unknown, key: string): Array<unknown> {
  const found = field(value, key);
  if (!Array.isArray(found)) {
    throw new Error(`${key} is not an array`);
  }
  return found;
}
