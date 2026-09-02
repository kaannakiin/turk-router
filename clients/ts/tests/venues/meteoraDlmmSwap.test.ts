import assert from "node:assert/strict";
import { test } from "node:test";

import type { Address } from "@solana/addresses";

import { assertTailLength, HopKind, programs, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { EVENT_AUTHORITY, MAX_BINS, PROGRAM_ADDRESS, resolve } = venues.meteoraDlmmSwap;

function accounts(
  overrides: Partial<venues.meteoraDlmmSwap.MeteoraDlmmSwapAccounts> = {},
): venues.meteoraDlmmSwap.MeteoraDlmmSwapAccounts {
  return {
    lbPair: key(1),
    binArrayBitmapExtension: undefined,
    reserveX: key(2),
    reserveY: key(3),
    userTokenIn: key(4),
    userTokenOut: key(5),
    mintX: key(6),
    mintY: key(7),
    oracle: key(8),
    hostFeeIn: undefined,
    user: key(9),
    ...overrides,
  };
}

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
  assert.equal(EVENT_AUTHORITY, "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");
});

test("account count tracks the bin tail length", () => {
  const one = resolve(accounts(), [key(10)]);
  assert.equal(one.accountCount, 17);
  assert.equal(one.accounts.length, 17);
  assert.equal(one.hopKind, HopKind.MeteoraDlmmSwap);
  const bins: ReadonlyArray<Address> = Array.from({ length: MAX_BINS }, () => key(20));
  assertTailLength(bins, 1, MAX_BINS);
  const max = resolve(accounts(), bins);
  assert.equal(max.accountCount, 24);
  assert.equal(max.accounts.length, 24);
});

test("account count does not depend on the optional slots", () => {
  const bare = resolve(accounts(), [key(10)]);
  const full = resolve(accounts({ binArrayBitmapExtension: key(11), hostFeeIn: key(12) }), [
    key(10),
  ]);
  assert.equal(bare.accountCount, full.accountCount);
});

test("slot zero and the repeated program slot are the venue program", () => {
  const built = resolve(accounts(), [key(10)]);
  for (const index of [0, 15]) {
    const meta = slot(built, index);
    assert.equal(meta.address, PROGRAM_ADDRESS);
    assert.ok(!writable(meta) && !signer(meta));
  }
});

test("absent optionals fall back to the program id sentinel", () => {
  const built = resolve(accounts(), [key(10)]);
  for (const index of [2, 10]) {
    assert.equal(slot(built, index).address, PROGRAM_ADDRESS);
    assert.ok(!writable(slot(built, index)));
  }
});

test("present optionals are writable", () => {
  const built = resolve(accounts({ binArrayBitmapExtension: key(11), hostFeeIn: key(12) }), [
    key(10),
  ]);
  assert.equal(slot(built, 2).address, key(11));
  assert.ok(writable(slot(built, 2)));
  assert.equal(slot(built, 10).address, key(12));
  assert.ok(writable(slot(built, 10)));
});

test("both token program slots are fixed to the token program", () => {
  const built = resolve(accounts(), [key(10)]);
  assert.equal(slot(built, 12).address, programs.TOKEN_PROGRAM_ADDRESS);
  assert.equal(slot(built, 13).address, programs.TOKEN_PROGRAM_ADDRESS);
  assert.equal(slot(built, 14).address, EVENT_AUTHORITY);
});

test("user is a readonly signer", () => {
  const user = slot(resolve(accounts(), [key(10)]), 11);
  assert.equal(user.address, key(9));
  assert.ok(signer(user) && !writable(user));
});
