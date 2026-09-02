import assert from "node:assert/strict";
import { test } from "node:test";

import { HopKind, programs, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { AMM_AUTHORITY, PROGRAM_ADDRESS, resolve } = venues.raydiumAmmV4;

function window(): ReturnType<typeof resolve> {
  return resolve({
    pool: key(1),
    baseVault: key(2),
    quoteVault: key(3),
    userSource: key(4),
    userDestination: key(5),
    payer: key(6),
  });
}

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
  assert.equal(AMM_AUTHORITY, "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1");
});

test("the window always declares nine accounts", () => {
  const built = window();
  assert.equal(built.accountCount, 9);
  assert.equal(built.accounts.length, 9);
  assert.equal(built.hopKind, HopKind.RaydiumAmmV4);
});

test("the slots carry the documented flags in order", () => {
  const built = window();
  const expected = [
    [PROGRAM_ADDRESS, false, false],
    [programs.TOKEN_PROGRAM_ADDRESS, false, false],
    [key(1), true, false],
    [AMM_AUTHORITY, false, false],
    [key(2), true, false],
    [key(3), true, false],
    [key(4), true, false],
    [key(5), true, false],
    [key(6), false, true],
  ] as const;
  expected.forEach(([address, isWritable, isSigner], index) => {
    const meta = slot(built, index);
    assert.equal(meta.address, address, `slot ${String(index)}`);
    assert.equal(writable(meta), isWritable, `slot ${String(index)} writable`);
    assert.equal(signer(meta), isSigner, `slot ${String(index)} signer`);
  });
});
