/**
 * Raydium AMM v4, `swap_base_in`. The window is always 9 accounts:
 * `[0]` program (readonly), `[1]` Token program (readonly), `[2]` pool (writable),
 * `[3]` AMM authority (readonly), `[4]` base vault (writable), `[5]` quote vault (writable),
 * `[6]` user source (writable), `[7]` user destination (writable), `[8]` payer (signer).
 * The classic Token program only.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import { TOKEN_PROGRAM_ADDRESS } from "../programs.js";
import { VenueWindow, readonlyAccount, readonlySignerAccount, writableAccount } from "./window.js";

/** The Raydium AMM v4 program. */
export const PROGRAM_ADDRESS: Address = address("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

/** The pool authority PDA every Raydium AMM v4 pool shares. */
export const AMM_AUTHORITY: Address = address("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1");

export interface RaydiumAmmV4Accounts {
  /** The pool account, owned by the program. */
  readonly pool: Address;
  readonly baseVault: Address;
  readonly quoteVault: Address;
  /** The caller's token account this hop debits. */
  readonly userSource: Address;
  /** The caller's token account this hop credits. */
  readonly userDestination: Address;
  /** The wallet authorizing the debit from `userSource`. */
  readonly payer: Address;
}

export function resolve(accounts: RaydiumAmmV4Accounts): VenueWindow {
  return new VenueWindow(HopKind.RaydiumAmmV4, [
    readonlyAccount(PROGRAM_ADDRESS),
    readonlyAccount(TOKEN_PROGRAM_ADDRESS),
    writableAccount(accounts.pool),
    readonlyAccount(AMM_AUTHORITY),
    writableAccount(accounts.baseVault),
    writableAccount(accounts.quoteVault),
    writableAccount(accounts.userSource),
    writableAccount(accounts.userDestination),
    readonlySignerAccount(accounts.payer),
  ]);
}
