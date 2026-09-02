import assert from "node:assert/strict";
import { test } from "node:test";

import type { Address } from "@solana/addresses";

import { assertTailLength, HopKind, programs, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { MAX_BIN_ARRAYS, PROGRAM_ADDRESS, resolve } = venues.meteoraDlmmSwap2;

function accounts(
  overrides: Partial<venues.meteoraDlmmSwap2.MeteoraDlmmSwap2Accounts> = {},
): venues.meteoraDlmmSwap2.MeteoraDlmmSwap2Accounts {
  return {
    pool: key(1),
    binArrayBitmapExtension: undefined,
    reserveX: key(2),
    reserveY: key(3),
    userTokenIn: key(4),
    userTokenOut: key(5),
    tokenXMint: key(6),
    tokenYMint: key(7),
    oracle: key(8),
    hostFeeIn: undefined,
    user: key(9),
    tokenXProgram: key(10),
    tokenYProgram: key(11),
    ...overrides,
  };
}

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
  const built = resolve(accounts(), [key(40)]);
  assert.equal(slot(built, 14).address, programs.MEMO_PROGRAM_ADDRESS);
  assert.equal(slot(built, 15).address, "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");
});

test("account count spans the bin tail range", () => {
  for (let length = 1; length <= MAX_BIN_ARRAYS; length += 1) {
    const binArrays: ReadonlyArray<Address> = Array.from({ length }, (_, index) => key(20 + index));
    assertTailLength(binArrays, 1, MAX_BIN_ARRAYS);
    const built = resolve(accounts(), binArrays);
    assert.equal(built.accountCount, 17 + length);
    assert.equal(built.accounts.length, 17 + length);
    assert.equal(built.hopKind, HopKind.MeteoraDlmmSwap2);
  }
});

test("the optional slots do not change account count", () => {
  const bare = resolve(accounts(), [key(30)]).accountCount;
  const full = resolve(accounts({ binArrayBitmapExtension: key(31), hostFeeIn: key(32) }), [
    key(30),
  ]).accountCount;
  assert.equal(bare, full);
});

test("slot 0 and slot 16 are the venue program readonly", () => {
  const built = resolve(accounts(), [key(40)]);
  for (const index of [0, 16]) {
    const meta = slot(built, index);
    assert.equal(meta.address, PROGRAM_ADDRESS);
    assert.ok(!writable(meta) && !signer(meta));
  }
});

test("absent optionals fall back to the program id sentinel", () => {
  const built = resolve(accounts(), [key(41)]);
  for (const index of [2, 10]) {
    assert.equal(slot(built, index).address, PROGRAM_ADDRESS);
    assert.ok(!writable(slot(built, index)));
  }
});

test("a named host fee account is writable", () => {
  const built = resolve(accounts({ hostFeeIn: key(32) }), [key(50)]);
  const hostFee = slot(built, 10);
  assert.equal(hostFee.address, key(32));
  assert.ok(writable(hostFee) && !signer(hostFee));
});

test("the caller's token programs sit at twelve and thirteen and user signs at eleven", () => {
  const built = resolve(accounts(), [key(50)]);
  assert.equal(slot(built, 12).address, key(10));
  assert.equal(slot(built, 13).address, key(11));
  assert.ok(signer(slot(built, 11)) && !writable(slot(built, 11)));
});
