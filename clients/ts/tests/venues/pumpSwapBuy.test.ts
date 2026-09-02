import assert from "node:assert/strict";
import { test } from "node:test";

import { HopKind, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { FEE_CONFIG, FEE_PROGRAM, GLOBAL_VOLUME_ACCUMULATOR, PROGRAM_ADDRESS, resolve } =
  venues.pumpSwapBuy;

function accounts(
  overrides: Partial<venues.pumpSwapBuy.PumpSwapBuyAccounts> = {},
): venues.pumpSwapBuy.PumpSwapBuyAccounts {
  return {
    pool: key(1),
    user: key(2),
    forwardedBeforeBaseMint: key(3),
    baseMint: key(4),
    quoteMint: key(5),
    baseTokenAccount: key(6),
    quoteTokenAccount: key(7),
    baseVault: key(8),
    quoteVault: key(9),
    forwardedBeforeVolumeAccumulator: [
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
    userVolumeAccumulator: key(20),
    forwardedClose: [key(21), key(22)],
    poolV2: undefined,
    cashback: undefined,
    ...overrides,
  };
}

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
  assert.equal(GLOBAL_VOLUME_ACCUMULATOR, "C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw");
  assert.equal(FEE_CONFIG, "5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");
  assert.equal(FEE_PROGRAM, "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
});

test("the base window declares twenty six accounts", () => {
  const built = resolve(accounts());
  assert.equal(built.accountCount, 26);
  assert.equal(built.accounts.length, 26);
  assert.equal(built.hopKind, HopKind.PumpSwapBuy);
  assert.equal(slot(built, 20).address, GLOBAL_VOLUME_ACCUMULATOR);
  assert.equal(slot(built, 21).address, key(20));
  assert.ok(writable(slot(built, 21)));
  assert.equal(slot(built, 22).address, FEE_CONFIG);
  assert.equal(slot(built, 23).address, FEE_PROGRAM);
});

test("each optional account grows the window by one", () => {
  assert.equal(resolve(accounts({ poolV2: key(30) })).accountCount, 27);
  assert.equal(resolve(accounts({ cashback: key(31) })).accountCount, 27);
  const both = resolve(accounts({ poolV2: key(30), cashback: key(31) }));
  assert.equal(both.accountCount, 28);
  assert.equal(both.accounts.length, 28);
});

test("the leading slot is the program readonly", () => {
  const program = slot(resolve(accounts()), 0);
  assert.equal(program.address, PROGRAM_ADDRESS);
  assert.ok(!writable(program) && !signer(program));
});

test("the user slot is writable and signed", () => {
  const user = slot(resolve(accounts()), 2);
  assert.equal(user.address, key(2));
  assert.ok(writable(user) && signer(user));
});

test("the fixed prefix carries the documented writable flags", () => {
  const built = resolve(accounts());
  const writableSlots = [1, 2, 6, 7, 8, 9, 11, 18, 21];
  built.accounts.slice(0, 24).forEach((meta, index) => {
    assert.equal(writable(meta), writableSlots.includes(index), `slot ${String(index)}`);
    assert.equal(signer(meta), index === 2, `slot ${String(index)}`);
  });
});

test("the optionals precede the closing pair cashback first", () => {
  const both = resolve(accounts({ cashback: key(31), poolV2: key(30) }));
  assert.equal(both.accounts.length, 28);
  assert.deepEqual([slot(both, 24).address, writable(slot(both, 24))], [key(31), true]);
  assert.deepEqual([slot(both, 25).address, writable(slot(both, 25))], [key(30), false]);
  assert.deepEqual([slot(both, 26).address, writable(slot(both, 26))], [key(21), false]);
  assert.deepEqual([slot(both, 27).address, writable(slot(both, 27))], [key(22), true]);

  const onlyPoolV2 = resolve(accounts({ poolV2: key(30) }));
  assert.equal(onlyPoolV2.accounts.length, 27);
  assert.deepEqual(
    [slot(onlyPoolV2, 24).address, writable(slot(onlyPoolV2, 24))],
    [key(30), false],
  );
  assert.equal(slot(onlyPoolV2, 25).address, key(21));
  assert.equal(slot(onlyPoolV2, 26).address, key(22));
});
