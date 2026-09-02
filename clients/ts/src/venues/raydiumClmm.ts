/**
 * Raydium CLMM, `swap_v2`. The window is 15 to 21 accounts: 14 fixed slots plus a tail of 1 to 7.
 * `[0]` program (readonly), `[1]` payer (signer), `[2]` amm_config (readonly), `[3]` pool
 * (writable), `[4]` input_token_account (writable), `[5]` output_token_account (writable),
 * `[6]` input_vault (writable), `[7]` output_vault (writable), `[8]` observation_state
 * (writable), `[9]` Token program (readonly), `[10]` Token Extensions program (readonly),
 * `[11]` Memo program (readonly), `[12]` input_mint (readonly), `[13]` output_mint (readonly),
 * `[14..]` the tail (writable): the optional tick-array bitmap extension, then the tick arrays
 * the swap will cross. Slots 9 and 10 are fixed whatever standard the mints use.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import {
  MEMO_PROGRAM_ADDRESS,
  TOKEN_2022_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
} from "../programs.js";
import { assertTailLength, type TupleRange } from "./tail.js";
import { VenueWindow, readonlyAccount, readonlySignerAccount, writableAccount } from "./window.js";

/** The Raydium CLMM program. */
export const PROGRAM_ADDRESS: Address = address("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

const TAIL_MIN = 1;
const TAIL_MAX = 7;

/**
 * The pool's variable tail: the optional tick-array bitmap extension, if the pool has one, then
 * the tick arrays the swap will cross, one to six.
 */
export type ClmmTail = TupleRange<Address, typeof TAIL_MIN, typeof TAIL_MAX>;

export interface RaydiumClmmAccounts {
  /** The signer whose token accounts the swap moves through. */
  readonly payer: Address;
  readonly ammConfig: Address;
  readonly pool: Address;
  readonly inputTokenAccount: Address;
  readonly outputTokenAccount: Address;
  readonly inputVault: Address;
  readonly outputVault: Address;
  readonly observationState: Address;
  readonly inputMint: Address;
  readonly outputMint: Address;
}

export function resolve(accounts: RaydiumClmmAccounts, tail: ClmmTail): VenueWindow {
  assertTailLength(tail, TAIL_MIN, TAIL_MAX);
  return new VenueWindow(HopKind.RaydiumClmm, [
    readonlyAccount(PROGRAM_ADDRESS),
    readonlySignerAccount(accounts.payer),
    readonlyAccount(accounts.ammConfig),
    writableAccount(accounts.pool),
    writableAccount(accounts.inputTokenAccount),
    writableAccount(accounts.outputTokenAccount),
    writableAccount(accounts.inputVault),
    writableAccount(accounts.outputVault),
    writableAccount(accounts.observationState),
    readonlyAccount(TOKEN_PROGRAM_ADDRESS),
    readonlyAccount(TOKEN_2022_PROGRAM_ADDRESS),
    readonlyAccount(MEMO_PROGRAM_ADDRESS),
    readonlyAccount(accounts.inputMint),
    readonlyAccount(accounts.outputMint),
    ...tail.map(writableAccount),
  ]);
}
