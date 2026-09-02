import assert from "node:assert/strict";
import { test } from "node:test";

import { AccountRole, isSignerRole, isWritableRole } from "@solana/instructions";

import { ALL_HOP_KINDS, hopKindName } from "../src/index.js";
import { parse, render, type Fixture, type Slot } from "./common/fixture.js";
import { resolve } from "./common/harness/index.js";
import { fixturePaths, readText } from "./common/paths.js";

function fixtures(): Array<[string, Fixture]> {
  return fixturePaths().map((path) => [path, parse(readText(path))]);
}

test("every fixture round-trips byte-exactly", () => {
  for (const path of fixturePaths()) {
    const text = readText(path);
    const rendered = render(parse(text));
    if (rendered !== text) {
      const original = text.split("\n");
      const differing = rendered.split("\n").findIndex((line, index) => line !== original[index]);
      assert.fail(`${path}: first difference at line ${String(differing + 1)}`);
    }
  }
});

test("the bundle covers every menu kind", () => {
  const seen = new Set(fixtures().map(([, fixture]) => fixture.hopKind));
  for (const kind of ALL_HOP_KINDS) {
    assert.ok(seen.has(kind), `no fixture for ${hopKindName(kind)}`);
  }
});

test("the kind label does not identify the venue", () => {
  const kindsByLabel = new Map<string, Set<number>>();
  for (const [, fixture] of fixtures()) {
    const kinds = kindsByLabel.get(fixture.kind) ?? new Set<number>();
    kinds.add(fixture.hopKind);
    kindsByLabel.set(fixture.kind, kinds);
  }
  assert.ok(
    [...kindsByLabel.values()].some((kinds) => kinds.size > 1),
    "some label spans two hop kinds",
  );
});

// The capture's fee payer and a program slot the capturing transaction marked writable are traces
// of that transaction, not of the venue; the Rust suite skips the same two.
function captureOnlyWritable(slot: Slot, fixture: Fixture): boolean {
  return slot.role === "payer" || slot.pubkey === fixture.programId;
}

test("every window a module builds matches its fixture", () => {
  for (const [path, fixture] of fixtures()) {
    assert.deepEqual(fixture.hookLens, [0, 0], `${path}: this package builds no hook groups`);
    const window = resolve(fixture);
    const metas = window.accounts;
    assert.equal(metas.length, fixture.slots.length + 1, `${path}: window length`);
    assert.equal(window.accountCount, metas.length, `${path}: declared count`);
    assert.equal(window.hopKind, fixture.hopKind, `${path}: hop kind`);

    const program = metas[0];
    assert.ok(program !== undefined);
    assert.equal(program.address, fixture.programId, `${path}: slot 0 is the venue program`);
    assert.equal(program.role, AccountRole.READONLY, `${path}: slot 0 flags`);

    fixture.slots.forEach((slot, index) => {
      const meta = metas[index + 1];
      assert.ok(meta !== undefined);
      const where = `${path}: slot ${String(index + 1)}`;
      assert.equal(meta.address, slot.pubkey, `${where} address`);
      assert.equal(isSignerRole(meta.role), slot.signer, `${where} signer`);
      if (!captureOnlyWritable(slot, fixture)) {
        assert.equal(isWritableRole(meta.role), slot.writable, `${where} writable`);
      }
    });
  }
});
