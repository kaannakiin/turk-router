import assert from "node:assert/strict";
import { test } from "node:test";

import { programs } from "../src/index.js";
import { findAssociatedTokenAddress, findConfigAccountAddress } from "../src/pda.js";
import { key } from "./common/keys.js";

// Computed once and cross-checked against Rust's Pubkey::find_program_address([b"config"]) — the
// golden corpus carries the same address at slot 4 of every positive case.
const CONFIG_ACCOUNT = "ahwevQHeGukiyNfPJVTpcmZ6RVX6xWNMANj5AXfSk23";

test("the config account is the pinned address and is stable across calls", async () => {
  assert.equal(await findConfigAccountAddress(), CONFIG_ACCOUNT);
  assert.equal(await findConfigAccountAddress(), CONFIG_ACCOUNT);
});

test("a wallet's token account differs per token program", async () => {
  const wallet = key(7);
  const mint = key(9);
  assert.notEqual(
    await findAssociatedTokenAddress(wallet, mint, programs.TOKEN_PROGRAM_ADDRESS),
    await findAssociatedTokenAddress(wallet, mint, programs.TOKEN_2022_PROGRAM_ADDRESS),
  );
});
