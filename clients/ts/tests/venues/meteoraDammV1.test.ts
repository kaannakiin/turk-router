import assert from "node:assert/strict";
import { test } from "node:test";

import { HopKind, programs, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { DYNAMIC_VAULT_PROGRAM_ADDRESS, PROGRAM_ADDRESS, resolve } = venues.meteoraDammV1;

function window(): ReturnType<typeof resolve> {
  return resolve({
    pool: key(1),
    userSource: key(2),
    userDest: key(3),
    aVault: key(4),
    bVault: key(5),
    aTokenVault: key(6),
    bTokenVault: key(7),
    aVaultLpMint: key(8),
    bVaultLpMint: key(9),
    aVaultLp: key(10),
    bVaultLp: key(11),
    protocolTokenFee: key(12),
    payer: key(13),
  });
}

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");
  assert.equal(DYNAMIC_VAULT_PROGRAM_ADDRESS, "24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi");
});

test("the window always declares sixteen accounts", () => {
  const built = window();
  assert.equal(built.accountCount, 16);
  assert.equal(built.accounts.length, 16);
  assert.equal(built.hopKind, HopKind.MeteoraDammV1);
});

test("the slots carry the documented flags in order", () => {
  const built = window();
  const addresses = [
    PROGRAM_ADDRESS,
    ...Array.from({ length: 13 }, (_, index) => key(index + 1)),
    DYNAMIC_VAULT_PROGRAM_ADDRESS,
    programs.TOKEN_PROGRAM_ADDRESS,
  ];
  built.accounts.forEach((meta, index) => {
    assert.equal(meta.address, addresses[index], `slot ${String(index)}`);
    assert.equal(writable(meta), index >= 1 && index <= 12, `slot ${String(index)} writable`);
    assert.equal(signer(meta), index === 13, `slot ${String(index)} signer`);
  });
  assert.equal(slot(built, 13).address, key(13));
});
