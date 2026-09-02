/**
 * Meteora DLMM, `swap2` — the form that accepts a Token Extensions mint on either side. The
 * window is 18 to 25 accounts: 17 fixed slots plus 1 to 8 bin arrays.
 * `[0]` program (readonly), `[1]` pool (writable), `[2]` bin_array_bitmap_extension (writable, or
 * the program as a readonly sentinel), `[3]` reserve_x (writable), `[4]` reserve_y (writable),
 * `[5]` user_token_in (writable), `[6]` user_token_out (writable), `[7]` token_x_mint (readonly),
 * `[8]` token_y_mint (readonly), `[9]` oracle (writable), `[10]` host_fee_in (writable, or the
 * sentinel), `[11]` user (signer), `[12]` token_x_program (readonly), `[13]` token_y_program
 * (readonly), `[14]` Memo program (readonly), `[15]` event authority (readonly), `[16]` program
 * again (readonly), `[17..]` bin arrays (writable). The program accepts transfer-hook account
 * groups on this kind; this module builds none.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import { MEMO_PROGRAM_ADDRESS } from "../programs.js";
import { assertTailLength, type TupleRange } from "./tail.js";
import {
  VenueWindow,
  readonlyAccount,
  readonlySignerAccount,
  writableAccount,
  writableOrSentinel,
} from "./window.js";

/** The Meteora DLMM program. */
export const PROGRAM_ADDRESS: Address = address("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

const EVENT_AUTHORITY: Address = address("D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");

const MIN_BIN_ARRAYS = 1;

/** Most bin arrays a `swap2` window's variable tail carries. */
export const MAX_BIN_ARRAYS = 8;

/** The bin array tail: 1 to `MAX_BIN_ARRAYS` accounts, in the order the instruction reads them. */
export type BinArrayTail = TupleRange<Address, typeof MIN_BIN_ARRAYS, typeof MAX_BIN_ARRAYS>;

export interface MeteoraDlmmSwap2Accounts {
  readonly pool: Address;
  /** The pair's bin array bitmap extension, when its active bin range needs one. */
  readonly binArrayBitmapExtension: Address | undefined;
  readonly reserveX: Address;
  readonly reserveY: Address;
  readonly userTokenIn: Address;
  readonly userTokenOut: Address;
  readonly tokenXMint: Address;
  readonly tokenYMint: Address;
  readonly oracle: Address;
  /** The host fee token account, when the caller names one. */
  readonly hostFeeIn: Address | undefined;
  /** The wallet authorizing the swap. */
  readonly user: Address;
  readonly tokenXProgram: Address;
  readonly tokenYProgram: Address;
}

export function resolve(accounts: MeteoraDlmmSwap2Accounts, binArrays: BinArrayTail): VenueWindow {
  assertTailLength(binArrays, MIN_BIN_ARRAYS, MAX_BIN_ARRAYS);
  return new VenueWindow(HopKind.MeteoraDlmmSwap2, [
    readonlyAccount(PROGRAM_ADDRESS),
    writableAccount(accounts.pool),
    writableOrSentinel(accounts.binArrayBitmapExtension, PROGRAM_ADDRESS),
    writableAccount(accounts.reserveX),
    writableAccount(accounts.reserveY),
    writableAccount(accounts.userTokenIn),
    writableAccount(accounts.userTokenOut),
    readonlyAccount(accounts.tokenXMint),
    readonlyAccount(accounts.tokenYMint),
    writableAccount(accounts.oracle),
    writableOrSentinel(accounts.hostFeeIn, PROGRAM_ADDRESS),
    readonlySignerAccount(accounts.user),
    readonlyAccount(accounts.tokenXProgram),
    readonlyAccount(accounts.tokenYProgram),
    readonlyAccount(MEMO_PROGRAM_ADDRESS),
    readonlyAccount(EVENT_AUTHORITY),
    readonlyAccount(PROGRAM_ADDRESS),
    ...binArrays.map(writableAccount),
  ]);
}
