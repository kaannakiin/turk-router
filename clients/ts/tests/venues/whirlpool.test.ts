import assert from "node:assert/strict";
import { test } from "node:test";

import type { Address } from "@solana/addresses";

import { assertTailLength, HopKind, isFindRouteError, programs, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { PROGRAM_ADDRESS, resolve } = venues.whirlpool;

function accounts(): venues.whirlpool.WhirlpoolAccounts {
  return {
    tokenProgramA: key(1),
    tokenProgramB: key(2),
    tokenAuthority: key(3),
    whirlpool: key(4),
    mintA: key(5),
    mintB: key(6),
    tokenOwnerAccountA: key(7),
    tokenVaultA: key(8),
    tokenOwnerAccountB: key(9),
    tokenVaultB: key(10),
    tickArray0: key(11),
    tickArray1: key(12),
    tickArray2: key(13),
    oracle: key(14),
  };
}

test("program id is the documented address", () => {
  assert.equal(PROGRAM_ADDRESS, "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
});

test("account count grows with the supplemental tail", () => {
  assert.equal(resolve(accounts(), []).accountCount, 16);
  assert.equal(resolve(accounts(), [key(20)]).accountCount, 17);
  assert.equal(resolve(accounts(), [key(20), key(21)]).accountCount, 18);
  const three = resolve(accounts(), [key(20), key(21), key(22)]);
  assert.equal(three.accountCount, 19);
  assert.equal(three.accounts.length, 19);
  assert.equal(three.hopKind, HopKind.Whirlpool);
  assert.equal(slot(three, 18).address, key(22));
  assert.ok(writable(slot(three, 18)));
});

test("slot zero is the program readonly", () => {
  const program = slot(resolve(accounts(), []), 0);
  assert.equal(program.address, PROGRAM_ADDRESS);
  assert.ok(!writable(program) && !signer(program));
});

test("the memo program is fixed not caller supplied", () => {
  assert.equal(slot(resolve(accounts(), []), 3).address, programs.MEMO_PROGRAM_ADDRESS);
});

test("the token authority slot is signer and readonly", () => {
  const authority = slot(resolve(accounts(), []), 4);
  assert.equal(authority.address, key(3));
  assert.ok(signer(authority) && !writable(authority));
});

test("an untyped array of four supplemental tick arrays is refused at run time", () => {
  const four: ReadonlyArray<Address> = [key(1), key(2), key(3), key(4)];
  assert.throws(
    () => {
      assertTailLength(four, 0, 3);
    },
    (thrown: unknown) =>
      isFindRouteError(thrown, "TailLength") &&
      thrown.detail.given === 4 &&
      thrown.detail.min === 0 &&
      thrown.detail.max === 3,
  );
});
