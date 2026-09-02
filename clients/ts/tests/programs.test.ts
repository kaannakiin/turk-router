import assert from "node:assert/strict";
import { test } from "node:test";

import { programs } from "../src/index.js";

test("the addresses are the documented ones", () => {
  assert.deepEqual(
    [
      programs.TOKEN_PROGRAM_ADDRESS,
      programs.TOKEN_2022_PROGRAM_ADDRESS,
      programs.ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
      programs.MEMO_PROGRAM_ADDRESS,
      programs.INSTRUCTIONS_SYSVAR_ADDRESS,
    ],
    [
      "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
      "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
      "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
      "Sysvar1nstructions1111111111111111111111111",
    ],
  );
});
