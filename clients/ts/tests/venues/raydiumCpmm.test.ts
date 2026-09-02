import assert from "node:assert/strict";
import { test } from "node:test";

import { HopKind, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { AUTHORITY, PROGRAM_ADDRESS, resolve } = venues.raydiumCpmm;

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
  assert.equal(AUTHORITY, "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL");
});

test("the window declares fourteen accounts", () => {
  const built = resolve({
    user: key(1),
    ammConfig: key(2),
    pool: key(3),
    inputTokenAccount: key(4),
    outputTokenAccount: key(5),
    inputVault: key(6),
    outputVault: key(7),
    inputTokenProgram: key(8),
    outputTokenProgram: key(9),
    inputMint: key(10),
    outputMint: key(11),
    observationState: key(12),
  });
  assert.equal(built.accountCount, 14);
  assert.equal(built.accounts.length, 14);
  assert.equal(built.hopKind, HopKind.RaydiumCpmm);
  assert.equal(slot(built, 0).address, PROGRAM_ADDRESS);
  assert.ok(signer(slot(built, 1)) && !writable(slot(built, 1)));
  assert.equal(slot(built, 2).address, AUTHORITY);
  const writableSlots = [4, 5, 6, 7, 8, 13];
  built.accounts.forEach((meta, index) => {
    assert.equal(writable(meta), writableSlots.includes(index), `slot ${String(index)}`);
    assert.equal(signer(meta), index === 1, `slot ${String(index)}`);
  });
});
