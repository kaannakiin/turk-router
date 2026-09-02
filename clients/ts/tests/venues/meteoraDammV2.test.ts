import assert from "node:assert/strict";
import { test } from "node:test";

import type { Address } from "@solana/addresses";

import { HopKind, programs, venues } from "../../src/index.js";
import { key } from "../common/keys.js";
import { signer, slot, writable } from "../common/metas.js";

const { DammV2Form, EVENT_AUTHORITY, POOL_AUTHORITY, PROGRAM_ADDRESS, resolve } =
  venues.meteoraDammV2;

function accounts(
  referralTokenAccount: Address | undefined,
): venues.meteoraDammV2.MeteoraDammV2Accounts {
  return {
    pool: key(1),
    inputTokenAccount: key(2),
    outputTokenAccount: key(3),
    tokenAVault: key(4),
    tokenBVault: key(5),
    tokenAMint: key(6),
    tokenBMint: key(7),
    payer: key(8),
    tokenAProgram: key(9),
    tokenBProgram: key(10),
    referralTokenAccount,
  };
}

test("the addresses are the documented ones", () => {
  assert.equal(PROGRAM_ADDRESS, "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
  assert.equal(POOL_AUTHORITY, "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");
  assert.equal(EVENT_AUTHORITY, "3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet");
});

test("account count reflects the form", () => {
  assert.equal(resolve(accounts(key(11)), DammV2Form.Base).accountCount, 15);
  assert.equal(resolve(accounts(undefined), DammV2Form.Base).accountCount, 15);
  assert.equal(resolve(accounts(key(11)), DammV2Form.RateLimited).accountCount, 16);
  assert.equal(resolve(accounts(undefined), DammV2Form.RateLimited).accountCount, 16);
  assert.equal(resolve(accounts(undefined), DammV2Form.Base).hopKind, HopKind.MeteoraDammV2);
});

test("an absent referral account falls back to the program id", () => {
  assert.equal(slot(resolve(accounts(undefined), DammV2Form.Base), 12).address, PROGRAM_ADDRESS);
});

test("the rate limited form appends the instructions sysvar", () => {
  const built = resolve(accounts(undefined), DammV2Form.RateLimited);
  assert.equal(built.accounts.length, 16);
  const sysvar = slot(built, 15);
  assert.equal(sysvar.address, programs.INSTRUCTIONS_SYSVAR_ADDRESS);
  assert.ok(!writable(sysvar) && !signer(sysvar));
});

test("slot flags match the swap2 account list", () => {
  const built = resolve(accounts(key(11)), DammV2Form.Base);
  const writableSlots = [2, 3, 4, 5, 6, 12];
  built.accounts.forEach((meta, index) => {
    assert.equal(writable(meta), writableSlots.includes(index), `slot ${String(index)}`);
    assert.equal(signer(meta), index === 9, `slot ${String(index)}`);
  });
  assert.equal(slot(built, 1).address, POOL_AUTHORITY);
  assert.equal(slot(built, 13).address, EVENT_AUTHORITY);
  assert.equal(slot(built, 14).address, PROGRAM_ADDRESS);
});

test("a named referral account is writable", () => {
  const referral = key(77);
  const meta = slot(resolve(accounts(referral), DammV2Form.Base), 12);
  assert.equal(meta.address, referral);
  assert.ok(writable(meta) && !signer(meta));
});
