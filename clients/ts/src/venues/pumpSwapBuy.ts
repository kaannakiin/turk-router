/**
 * PumpSwap, buying the base token. The window is 26 accounts, plus 1 with `cashback` and 1 with
 * `poolV2`, up to 28:
 * `[0]` program (readonly), `[1]` pool (writable), `[2]` user (writable, signer),
 * `[3]` forwarded (readonly), `[4]` base_mint (readonly), `[5]` quote_mint (readonly),
 * `[6]` base_token_account (writable), `[7]` quote_token_account (writable), `[8]` base_vault
 * (writable), `[9]` quote_vault (writable), `[10..=19]` ten forwarded slots, writable at `[11]`
 * and `[18]`, `[20]` global volume accumulator (readonly), `[21]` user_volume_accumulator
 * (writable), `[22]` fee config (readonly), `[23]` fee program (readonly), then the tail: the
 * `cashback` account when present (writable), the `poolV2` account when present (readonly), and
 * the two closing accounts that end every window (readonly, then writable).
 * Forwarded slots are not read; their role names are withheld until checked against the venue's
 * published interface. The token program accounts are among them, so either token standard
 * works on either leg.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import type { TupleOfLength } from "./tail.js";
import { VenueWindow, readonlyAccount, writableAccount, writableSignerAccount } from "./window.js";

/** The PumpSwap AMM program. */
export const PROGRAM_ADDRESS: Address = address("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

/** The global volume accumulator every buy names at slot 20. */
export const GLOBAL_VOLUME_ACCUMULATOR: Address = address(
  "C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw",
);

/** The fee configuration account every buy names at slot 22. */
export const FEE_CONFIG: Address = address("5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");

/** The external fee program every buy names at slot 23. */
export const FEE_PROGRAM: Address = address("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");

export interface PumpSwapBuyAccounts {
  readonly pool: Address;
  /** The signer whose token accounts the swap moves through. */
  readonly user: Address;
  /** Window slot 3, forwarded without validation. */
  readonly forwardedBeforeBaseMint: Address;
  /** The mint the swap receives. */
  readonly baseMint: Address;
  /** The mint the swap spends. */
  readonly quoteMint: Address;
  readonly baseTokenAccount: Address;
  readonly quoteTokenAccount: Address;
  readonly baseVault: Address;
  readonly quoteVault: Address;
  /** Window slots 10 to 19, forwarded without validation. */
  readonly forwardedBeforeVolumeAccumulator: TupleOfLength<Address, 10>;
  readonly userVolumeAccumulator: Address;
  /** The two accounts that close every window: readonly, then writable. */
  readonly forwardedClose: readonly [Address, Address];
  /** The pool-v2 sibling account, present only for pools that name one. */
  readonly poolV2: Address | undefined;
  /** The cashback ledger account, present only for cashback pools. */
  readonly cashback: Address | undefined;
}

export function resolve(accounts: PumpSwapBuyAccounts): VenueWindow {
  const [f0, f1, f2, f3, f4, f5, f6, f7, f8, f9] = accounts.forwardedBeforeVolumeAccumulator;
  const [closeReadonly, closeWritable] = accounts.forwardedClose;
  return new VenueWindow(HopKind.PumpSwapBuy, [
    readonlyAccount(PROGRAM_ADDRESS),
    writableAccount(accounts.pool),
    writableSignerAccount(accounts.user),
    readonlyAccount(accounts.forwardedBeforeBaseMint),
    readonlyAccount(accounts.baseMint),
    readonlyAccount(accounts.quoteMint),
    writableAccount(accounts.baseTokenAccount),
    writableAccount(accounts.quoteTokenAccount),
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
    readonlyAccount(GLOBAL_VOLUME_ACCUMULATOR),
    writableAccount(accounts.userVolumeAccumulator),
    readonlyAccount(FEE_CONFIG),
    readonlyAccount(FEE_PROGRAM),
    ...(accounts.cashback === undefined ? [] : [writableAccount(accounts.cashback)]),
    ...(accounts.poolV2 === undefined ? [] : [readonlyAccount(accounts.poolV2)]),
    readonlyAccount(closeReadonly),
    writableAccount(closeWritable),
  ]);
}
