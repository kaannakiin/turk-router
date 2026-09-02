/**
 * Orca Whirlpool, `swap_v2`. The window is 16 accounts, plus one per supplemental tick array up
 * to 19:
 * `[0]` program (readonly), `[1]` token program A (readonly), `[2]` token program B (readonly),
 * `[3]` Memo program (readonly), `[4]` token authority (signer), `[5]` whirlpool (writable),
 * `[6]` mint A (readonly), `[7]` mint B (readonly), `[8]` user token account A (writable),
 * `[9]` vault A (writable), `[10]` user token account B (writable), `[11]` vault B (writable),
 * `[12]` tick array 0 (writable), `[13]` tick array 1 (writable), `[14]` tick array 2 (writable),
 * `[15]` oracle (writable), `[16..=18]` zero to three supplemental tick arrays (writable).
 * Token program A and B are each the Token program or the Token Extensions program.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import { MEMO_PROGRAM_ADDRESS } from "../programs.js";
import { assertTailLength, type TupleRange } from "./tail.js";
import { VenueWindow, readonlyAccount, readonlySignerAccount, writableAccount } from "./window.js";

/** The Orca Whirlpool program. */
export const PROGRAM_ADDRESS: Address = address("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

const SUPPLEMENTAL_MIN = 0;
const SUPPLEMENTAL_MAX = 3;

/**
 * Zero to three supplemental tick arrays, appended after the fixed 16 accounts. A four-element
 * literal is a type error; an array of unknown length is narrowed with `assertTailLength`.
 */
export type SupplementalTickArrays = TupleRange<
  Address,
  typeof SUPPLEMENTAL_MIN,
  typeof SUPPLEMENTAL_MAX
>;

export interface WhirlpoolAccounts {
  readonly tokenProgramA: Address;
  readonly tokenProgramB: Address;
  /** The wallet whose tokens the swap moves. Signs. */
  readonly tokenAuthority: Address;
  readonly whirlpool: Address;
  readonly mintA: Address;
  readonly mintB: Address;
  readonly tokenOwnerAccountA: Address;
  readonly tokenVaultA: Address;
  readonly tokenOwnerAccountB: Address;
  readonly tokenVaultB: Address;
  readonly tickArray0: Address;
  readonly tickArray1: Address;
  readonly tickArray2: Address;
  readonly oracle: Address;
}

export function resolve(
  accounts: WhirlpoolAccounts,
  supplemental: SupplementalTickArrays,
): VenueWindow {
  assertTailLength(supplemental, SUPPLEMENTAL_MIN, SUPPLEMENTAL_MAX);
  return new VenueWindow(HopKind.Whirlpool, [
    readonlyAccount(PROGRAM_ADDRESS),
    readonlyAccount(accounts.tokenProgramA),
    readonlyAccount(accounts.tokenProgramB),
    readonlyAccount(MEMO_PROGRAM_ADDRESS),
    readonlySignerAccount(accounts.tokenAuthority),
    writableAccount(accounts.whirlpool),
    readonlyAccount(accounts.mintA),
    readonlyAccount(accounts.mintB),
    writableAccount(accounts.tokenOwnerAccountA),
    writableAccount(accounts.tokenVaultA),
    writableAccount(accounts.tokenOwnerAccountB),
    writableAccount(accounts.tokenVaultB),
    writableAccount(accounts.tickArray0),
    writableAccount(accounts.tickArray1),
    writableAccount(accounts.tickArray2),
    writableAccount(accounts.oracle),
    ...supplemental.map(writableAccount),
  ]);
}
