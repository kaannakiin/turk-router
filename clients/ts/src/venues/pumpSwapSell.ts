/**
 * PumpSwap, `sell` (base mint to quote mint). The window is 24 accounts, plus 2 with `cashback`
 * and 1 with `poolV2`, up to 27:
 * `[0]` program (readonly), `[1]` pool (writable), `[2]` user (writable, signer),
 * `[3]` forwarded (readonly), `[4]` base_mint (readonly), `[5]` quote_mint (readonly),
 * `[6]` base_ata (writable), `[7]` quote_ata (writable), `[8]` base_vault (writable),
 * `[9]` quote_vault (writable), `[10..=19]` ten forwarded slots, writable at `[11]` and `[18]`,
 * `[20]` fee config (readonly), `[21]` fee program (readonly), then the tail: the two `cashback`
 * accounts when present (writable), the `poolV2` account when present (readonly), and the two
 * closing accounts that end every window (readonly, then writable).
 * Slots 3 and 10 to 19 are forwarded without being read; their role names are withheld until
 * checked against the venue's published interface. Positions `[12]` and `[13]` carry the token
 * programs for `baseMint` and `quoteMint`.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import type { TupleOfLength } from "./tail.js";
import { VenueWindow, readonlyAccount, writableAccount, writableSignerAccount } from "./window.js";

/** The PumpSwap AMM program. */
export const PROGRAM_ADDRESS: Address = address("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

/** The fee-config PDA every pool shares. */
export const FEE_CONFIG: Address = address("5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");

/** The external fee program every pool shares. */
export const FEE_PROGRAM: Address = address("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");

export interface PumpSwapSellAccounts {
  readonly pool: Address;
  /** The wallet authorizing the swap. Signs. */
  readonly user: Address;
  /** Window position `[3]`. */
  readonly forwardedBeforeBaseMint: Address;
  readonly baseMint: Address;
  readonly quoteMint: Address;
  /** The caller's token account for `baseMint`, the sell's input. */
  readonly baseAta: Address;
  /** The caller's token account for `quoteMint`, the sell's output. */
  readonly quoteAta: Address;
  readonly baseVault: Address;
  readonly quoteVault: Address;
  /** Window positions `[10]` through `[19]`, in order. */
  readonly forwardedBeforeFeeConfig: TupleOfLength<Address, 10>;
  /** The pool's cashback volume-ledger pair, present only on a cashback pool. */
  readonly cashback: readonly [Address, Address] | undefined;
  /** The pool's `pool-v2` sidecar account, present only when the pool has a creator set. */
  readonly poolV2: Address | undefined;
  /** The two accounts that close every window: readonly, then writable. */
  readonly forwardedClose: readonly [Address, Address];
}

export function resolve(accounts: PumpSwapSellAccounts): VenueWindow {
  const [f0, f1, f2, f3, f4, f5, f6, f7, f8, f9] = accounts.forwardedBeforeFeeConfig;
  const [closeReadonly, closeWritable] = accounts.forwardedClose;
  return new VenueWindow(HopKind.PumpSwapSell, [
    readonlyAccount(PROGRAM_ADDRESS),
    writableAccount(accounts.pool),
    writableSignerAccount(accounts.user),
    readonlyAccount(accounts.forwardedBeforeBaseMint),
    readonlyAccount(accounts.baseMint),
    readonlyAccount(accounts.quoteMint),
    writableAccount(accounts.baseAta),
    writableAccount(accounts.quoteAta),
    writableAccount(accounts.baseVault),
    writableAccount(accounts.quoteVault),
    readonlyAccount(f0),
    writableAccount(f1),
    readonlyAccount(f2),
    readonlyAccount(f3),
    readonlyAccount(f4),
    readonlyAccount(f5),
    readonlyAccount(f6),
    readonlyAccount(f7),
    writableAccount(f8),
    readonlyAccount(f9),
    readonlyAccount(FEE_CONFIG),
    readonlyAccount(FEE_PROGRAM),
    ...(accounts.cashback === undefined ? [] : accounts.cashback.map(writableAccount)),
    ...(accounts.poolV2 === undefined ? [] : [readonlyAccount(accounts.poolV2)]),
    readonlyAccount(closeReadonly),
    writableAccount(closeWritable),
  ]);
}
