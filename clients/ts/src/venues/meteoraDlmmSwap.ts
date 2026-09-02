/**
 * Meteora DLMM, `swap` — the classic form, pinned to the Token program on both sides. The window
 * is 17 to 24 accounts: 16 fixed slots plus 1 to 8 bin arrays.
 * `[0]` program (readonly), `[1]` lb_pair (writable), `[2]` bin_array_bitmap_extension
 * (writable, or the program as a readonly sentinel), `[3]` reserve_x (writable), `[4]` reserve_y
 * (writable), `[5]` user_token_in (writable), `[6]` user_token_out (writable), `[7]` mint_x
 * (readonly), `[8]` mint_y (readonly), `[9]` oracle (writable), `[10]` host_fee_in (writable, or
 * the sentinel), `[11]` user (signer), `[12]` Token program (readonly), `[13]` Token program
 * (readonly), `[14]` event authority (readonly), `[15]` program again (readonly), `[16..]` bin
 * arrays (writable).
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import { TOKEN_PROGRAM_ADDRESS } from "../programs.js";
import { assertTailLength, type TupleRange } from "./tail.js";
import {
  VenueWindow,
  readonlyAccount,
  readonlySignerAccount,
  writableAccount,
  writableOrSentinel,
} from "./window.js";

/**
 * The Meteora DLMM program. Also the sentinel this module writes for `binArrayBitmapExtension`
 * and `hostFeeIn` when the caller has none.
 */
export const PROGRAM_ADDRESS: Address = address("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

/** The event-authority PDA this instruction reads. */
export const EVENT_AUTHORITY: Address = address("D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");

const MIN_BINS = 1;

/** The bin array tail's longest reach. */
export const MAX_BINS = 8;

/** The bin array tail: 1 to `MAX_BINS` accounts, in the order the instruction reads them. */
export type BinArrayTail = TupleRange<Address, typeof MIN_BINS, typeof MAX_BINS>;

export interface MeteoraDlmmSwapAccounts {
  readonly lbPair: Address;
  /** The pool's bin array bitmap extension, when it has one. */
  readonly binArrayBitmapExtension: Address | undefined;
  readonly reserveX: Address;
  readonly reserveY: Address;
  readonly userTokenIn: Address;
  readonly userTokenOut: Address;
  readonly mintX: Address;
  readonly mintY: Address;
  readonly oracle: Address;
  /** The host's fee-collection account, when the pool has a host fee configured. */
  readonly hostFeeIn: Address | undefined;
  /** The signer whose tokens move. */
  readonly user: Address;
}

export function resolve(accounts: MeteoraDlmmSwapAccounts, binArrays: BinArrayTail): VenueWindow {
  assertTailLength(binArrays, MIN_BINS, MAX_BINS);
  return new VenueWindow(HopKind.MeteoraDlmmSwap, [
    readonlyAccount(PROGRAM_ADDRESS),
    writableAccount(accounts.lbPair),
    writableOrSentinel(accounts.binArrayBitmapExtension, PROGRAM_ADDRESS),
    writableAccount(accounts.reserveX),
    writableAccount(accounts.reserveY),
    writableAccount(accounts.userTokenIn),
    writableAccount(accounts.userTokenOut),
    readonlyAccount(accounts.mintX),
    readonlyAccount(accounts.mintY),
    writableAccount(accounts.oracle),
    writableOrSentinel(accounts.hostFeeIn, PROGRAM_ADDRESS),
    readonlySignerAccount(accounts.user),
    readonlyAccount(TOKEN_PROGRAM_ADDRESS),
    readonlyAccount(TOKEN_PROGRAM_ADDRESS),
    readonlyAccount(EVENT_AUTHORITY),
    readonlyAccount(PROGRAM_ADDRESS),
    ...binArrays.map(writableAccount),
  ]);
}
