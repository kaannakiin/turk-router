import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ALL_HOP_KINDS,
  HopKind,
  hopKindFromByte,
  hopKindName,
  isFindRouteError,
  isHopKind,
} from "../src/index.js";
import { arrayField, field, manifest, stringField } from "./common/paths.js";

test("discriminants are dense from zero", () => {
  ALL_HOP_KINDS.forEach((kind, index) => {
    assert.equal(kind, index);
    assert.equal(hopKindFromByte(kind), kind);
    assert.ok(isHopKind(kind));
  });
  assert.equal(ALL_HOP_KINDS.length, 10);
});

test("bytes past the menu set are refused", () => {
  for (let raw = 10; raw <= 255; raw += 1) {
    assert.ok(!isHopKind(raw));
    assert.throws(
      () => hopKindFromByte(raw),
      (thrown: unknown) => isFindRouteError(thrown, "UnknownHopKind") && thrown.detail.raw === raw,
    );
  }
  for (const raw of [1.5, Number.NaN, -1, 256]) {
    assert.throws(
      () => hopKindFromByte(raw),
      (thrown: unknown) => isFindRouteError(thrown, "UnknownHopKind"),
    );
  }
});

test("the names are the manifest names", () => {
  const names = arrayField(field(manifest(), "find_route"), "menu_eligible_hop_kinds").map(
    (entry) => stringField(entry, "name"),
  );
  assert.deepEqual(
    ALL_HOP_KINDS.map((kind) => hopKindName(kind)),
    names,
  );
  assert.equal(HopKind.Whirlpool, 1);
});
