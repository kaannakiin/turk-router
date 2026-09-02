/**
 * Every wire number this package declares has to equal the one `wire/wire-manifest.json` carries.
 * The manifest is generated from the deployed program's own constants; this is the check that
 * keeps a hand-copied literal from drifting into a client. It needs no network and no secret.
 */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";

import { getBase16Decoder } from "@solana/codecs-strings";

import {
  ALL_BASE_MINTS,
  ALL_HOP_KINDS,
  HopKind,
  hopKindName,
  WIRE_EPOCH,
  wire,
} from "../src/index.js";
import { reachableAccountCounts } from "./common/harness/index.js";
import {
  arrayField,
  asArray,
  booleanField,
  field,
  manifest,
  numberField,
  stringField,
} from "./common/paths.js";

function findRoute(key: string): unknown {
  return field(field(manifest(), "find_route"), key);
}

test("the wire epoch matches the manifest", () => {
  assert.equal(numberField(manifest(), "wire_epoch"), WIRE_EPOCH);
});

test("the manifest carries every field the clients need", () => {
  const root = manifest();
  for (const key of ["wire_epoch", "program_id", "config_seed", "router_errors"]) {
    assert.notEqual(field(root, key), null, key);
  }
  for (const key of [
    "discriminator",
    "header_len",
    "menu_entry_len",
    "max_menu_pools",
    "max_route_mints",
    "max_menu_accounts",
    "max_hook_group_len",
    "flags",
    "prefix_accounts",
    "prefix_account_metas",
    "base_mints",
    "menu_eligible_hop_kinds",
  ]) {
    assert.notEqual(findRoute(key), null, key);
  }
});

test("the program and its config seed match the manifest", () => {
  assert.equal(stringField(manifest(), "program_id"), wire.ROUTER_PROGRAM_ADDRESS);
  assert.equal(stringField(manifest(), "config_seed"), wire.CONFIG_SEED);
});

test("the discriminator matches the manifest and its own name", () => {
  const declared = getBase16Decoder().decode(wire.FIND_ROUTE_DISC);
  assert.equal(declared, findRoute("discriminator"));
  const digest = createHash("sha256").update("global:find_route").digest("hex");
  assert.equal(declared, digest.slice(0, 16));
});

test("the widths and ceilings match the manifest", () => {
  for (const [key, value] of [
    ["header_len", wire.HEADER_LEN],
    ["menu_entry_len", wire.MENU_ENTRY_LEN],
    ["max_menu_pools", wire.MAX_MENU_POOLS],
    ["max_route_mints", wire.MAX_ROUTE_MINTS],
    ["max_menu_accounts", wire.MAX_MENU_ACCOUNTS],
    ["max_hook_group_len", wire.MAX_HOOK_GROUP_LEN],
  ] as const) {
    assert.equal(findRoute(key), value, key);
  }
});

test("the flag bits match the manifest", () => {
  const flags = findRoute("flags");
  assert.equal(numberField(flags, "flashloan"), wire.FLAG_FLASHLOAN);
  assert.equal(numberField(flags, "fail_if_no_profit"), wire.FLAG_FAIL_IF_NO_PROFIT);
});

test("the prefix matches the manifest names and flags", () => {
  assert.deepEqual(
    asArray(findRoute("prefix_accounts")),
    wire.PREFIX_ACCOUNT_METAS.map((meta) => meta.name),
  );
  const metas = asArray(findRoute("prefix_account_metas")).map((meta) => ({
    name: stringField(meta, "name"),
    isSigner: booleanField(meta, "is_signer"),
    isWritable: booleanField(meta, "is_writable"),
  }));
  assert.deepEqual(metas, [...wire.PREFIX_ACCOUNT_METAS]);
  assert.deepEqual(metas[5], { name: "fee_ata", isSigner: false, isWritable: true });
});

test("the base mints match the manifest in order", () => {
  assert.deepEqual(asArray(findRoute("base_mints")), [...ALL_BASE_MINTS]);
});

// Equality both ways: a length the module cannot build is liquidity the client cannot route, and
// a length it builds that the program refuses is an instruction that fails on landing.
test("every menu kind builds exactly the window lengths the program accepts", () => {
  const entries = asArray(findRoute("menu_eligible_hop_kinds"));
  const byValue = (a: number, b: number): number => a - b;
  const listed = entries.map((entry) => numberField(entry, "discriminant")).sort(byValue);
  assert.deepEqual(listed, [...ALL_HOP_KINDS].sort(byValue));
  for (const entry of entries) {
    const raw = numberField(entry, "discriminant");
    const kind = ALL_HOP_KINDS.find((candidate) => candidate === raw);
    assert.ok(kind !== undefined, `${String(raw)} is a menu kind`);
    assert.equal(stringField(entry, "name"), hopKindName(kind));
    const accepted = [...new Set(arrayField(entry, "window_lens").map(Number))].sort(
      (a, b) => a - b,
    );
    const buildable = [...new Set(reachableAccountCounts(kind))].sort((a, b) => a - b);
    assert.deepEqual(
      buildable,
      accepted,
      `${hopKindName(kind)}: account counts the module can declare`,
    );
  }
});

test("the hook capable kinds are the two this package documents as unsupported", () => {
  const capable = asArray(findRoute("menu_eligible_hop_kinds"))
    .filter((entry) => booleanField(entry, "hook_capable"))
    .map((entry) => numberField(entry, "discriminant"))
    .sort((a, b) => a - b);
  assert.deepEqual(capable, [HopKind.Whirlpool, HopKind.MeteoraDlmmSwap2]);
});

test("the error numbers are dense from six thousand", () => {
  const errors = arrayField(manifest(), "router_errors");
  assert.ok(errors.length > 0);
  errors.forEach((error, index) => {
    assert.equal(numberField(error, "code"), 6000 + index, stringField(error, "name"));
  });
});
