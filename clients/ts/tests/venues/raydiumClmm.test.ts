import assert from "node:assert/strict";
import { test } from "node:test";

import type { Address } from "@solana/addresses";

import { assertTailLength, HopKind, isFindRouteError, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { PROGRAM_ADDRESS, resolve } = venues.raydiumClmm;

function accounts(): venues.raydiumClmm.RaydiumClmmAccounts {
  return {
    payer: key(1),
    ammConfig: key(2),
    pool: key(3),
    inputTokenAccount: key(4),
    outputTokenAccount: key(5),
    inputVault: key(6),
    outputVault: key(7),
    observationState: key(8),
    inputMint: key(9),
    outputMint: key(10),
  };
}

test("the program id is the documented one", () => {
  assert.equal(PROGRAM_ADDRESS, "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");
});

test("account count is fourteen plus the tail length", () => {
  for (let length = 1; length <= 7; length += 1) {
    const tail: ReadonlyArray<Address> = Array.from({ length }, (_, index) => key(100 + index));
    assertTailLength(tail, 1, 7);
    const built = resolve(accounts(), tail);
    assert.equal(built.accountCount, 14 + length, `tail ${String(length)}`);
    assert.equal(built.accounts.length, 14 + length);
    assert.equal(built.hopKind, HopKind.RaydiumClmm);
    assert.ok(writable(slot(built, 14 + length - 1)));
  }
});

test("slot zero is the program readonly", () => {
  const program = slot(resolve(accounts(), [key(50)]), 0);
  assert.equal(program.address, PROGRAM_ADDRESS);
  assert.ok(!writable(program) && !signer(program));
});

test("the payer is the only signer", () => {
  const built = resolve(accounts(), [key(50)]);
  built.accounts.forEach((meta, index) => {
    assert.equal(signer(meta), index === 1, `slot ${String(index)}`);
  });
  assert.equal(slot(built, 1).address, key(1));
  assert.ok(!writable(slot(built, 1)));
});

test("an untyped oversized tail is refused at run time", () => {
  const eight: ReadonlyArray<Address> = Array.from({ length: 8 }, (_, index) => key(index + 1));
  assert.throws(
    () => {
      assertTailLength(eight, 1, 7);
    },
    (thrown: unknown) => isFindRouteError(thrown, "TailLength") && thrown.detail.given === 8,
  );
});
