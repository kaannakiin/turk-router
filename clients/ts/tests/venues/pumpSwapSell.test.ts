import assert from "node:assert/strict";
import { test } from "node:test";

import type { Address } from "@solana/addresses";

import { HopKind, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { FEE_CONFIG, FEE_PROGRAM, PROGRAM_ADDRESS, resolve } = venues.pumpSwapSell;

function accounts(
  cashback: readonly [Address, Address] | undefined,
  poolV2: Address | undefined,
): venues.pumpSwapSell.PumpSwapSellAccounts {
  return {
    pool: key(1),
    user: key(2),
    forwardedBeforeBaseMint: key(3),
    baseMint: key(4),
    quoteMint: key(5),
    baseAta: key(6),
    quoteAta: key(7),
    baseVault: key(8),
    quoteVault: key(9),
    forwardedBeforeFeeConfig: [
      key(10),
      key(11),
      key(12),
      key(13),
      key(14),
      key(15),
      key(16),
      key(17),
      key(18),
      key(19),
    ],
    cashback,
    poolV2,
    forwardedClose: [key(20), key(21)],
  };
}

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
  assert.equal(FEE_CONFIG, "5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");
  assert.equal(FEE_PROGRAM, "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
});

test("account count reflects the optional tail", () => {
  assert.equal(resolve(accounts(undefined, undefined)).accountCount, 24);
  assert.equal(resolve(accounts(undefined, key(30))).accountCount, 25);
  assert.equal(resolve(accounts([key(31), key(32)], undefined)).accountCount, 26);
  assert.equal(resolve(accounts([key(31), key(32)], key(30))).accountCount, 27);
});

test("every construction declares a count matching its accounts length", () => {
  for (const cashback of [undefined, [key(31), key(32)] as const]) {
    for (const poolV2 of [undefined, key(30)]) {
      const built = resolve(accounts(cashback, poolV2));
      assert.equal(built.accountCount, built.accounts.length);
      assert.equal(built.hopKind, HopKind.PumpSwapSell);
    }
  }
});

test("slot zero is the program readonly", () => {
  const program = slot(resolve(accounts(undefined, undefined)), 0);
  assert.equal(program.address, PROGRAM_ADDRESS);
  assert.ok(!writable(program) && !signer(program));
});

test("slot two is the only signer and carries user", () => {
  const built = resolve(accounts(undefined, undefined));
  built.accounts.forEach((meta, index) => {
    assert.equal(signer(meta), index === 2, `slot ${String(index)}`);
  });
  assert.equal(slot(built, 2).address, key(2));
  assert.ok(writable(slot(built, 2)));
});

test("the fixed prefix carries the documented writable flags", () => {
  const built = resolve(accounts(undefined, undefined));
  const writableSlots = [1, 2, 6, 7, 8, 9, 11, 18];
  built.accounts.slice(0, 22).forEach((meta, index) => {
    assert.equal(writable(meta), writableSlots.includes(index), `slot ${String(index)}`);
  });
});

test("fee config and fee program sit at twenty and twenty one", () => {
  const built = resolve(accounts(undefined, undefined));
  assert.equal(slot(built, 20).address, FEE_CONFIG);
  assert.equal(slot(built, 21).address, FEE_PROGRAM);
});

test("cashback precedes pool v2 which precedes the closing pair", () => {
  const built = resolve(accounts([key(31), key(32)], key(30)));
  assert.equal(built.accounts.length, 27);
  const expected = [
    [key(31), true],
    [key(32), true],
    [key(30), false],
    [key(20), false],
    [key(21), true],
  ] as const;
  expected.forEach(([address, isWritable], offset) => {
    const meta = slot(built, 22 + offset);
    assert.equal(meta.address, address, `slot ${String(22 + offset)}`);
    assert.equal(writable(meta), isWritable, `slot ${String(22 + offset)}`);
  });
});
