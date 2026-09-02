/** Program and sysvar addresses that several venue windows name in fixed positions. */
import { address, type Address } from "@solana/addresses";

/** The Token program. */
export const TOKEN_PROGRAM_ADDRESS: Address = address(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);

/** The Token Extensions program. */
export const TOKEN_2022_PROGRAM_ADDRESS: Address = address(
  "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
);

/** The Associated Token Account program. */
export const ASSOCIATED_TOKEN_PROGRAM_ADDRESS: Address = address(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
);

/** The Memo program. */
export const MEMO_PROGRAM_ADDRESS: Address = address("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

/** The instructions sysvar. */
export const INSTRUCTIONS_SYSVAR_ADDRESS: Address = address(
  "Sysvar1nstructions1111111111111111111111111",
);
