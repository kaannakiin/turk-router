/**
 * Meteora DAMM v2 (`cp-amm`), `swap2`. The window is 15 accounts, or 16 in the rate-limited
 * form, which appends the instructions sysvar:
 * `[0]` program (readonly), `[1]` pool authority (readonly), `[2]` pool (writable),
 * `[3]` input_token_account (writable), `[4]` output_token_account (writable), `[5]` token_a_vault
 * (writable), `[6]` token_b_vault (writable), `[7]` token_a_mint (readonly), `[8]` token_b_mint
 * (readonly), `[9]` payer (signer), `[10]` token_a_program (readonly), `[11]` token_b_program
 * (readonly), `[12]` referral_token_account (writable, or the program as a readonly sentinel),
 * `[13]` event authority (readonly), `[14]` program again (readonly), `[15]` instructions sysvar
 * (readonly, rate-limited form only). Each token program independently names the Token program
 * or the Token Extensions program.
 */
import { address, type Address } from "@solana/addresses";

import { assertNever } from "../assertNever.js";
import { HopKind } from "../hopKind.js";
import { INSTRUCTIONS_SYSVAR_ADDRESS } from "../programs.js";
import {
  VenueWindow,
  readonlyAccount,
  readonlySignerAccount,
  writableAccount,
  writableOrSentinel,
} from "./window.js";

/** The Meteora DAMM v2 program. */
export const PROGRAM_ADDRESS: Address = address("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

/** The pool authority PDA every pool shares. */
export const POOL_AUTHORITY: Address = address("HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");

/** The event-CPI authority PDA every pool shares. */
export const EVENT_AUTHORITY: Address = address("3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet");

/** Whether the window carries the instructions sysvar a rate-limiter pool requires. */
export const DammV2Form = {
  /** The 15-account window: no rate limiter. */
  Base: "Base",
  /** The 16-account window: the instructions sysvar follows the program. */
  RateLimited: "RateLimited",
} as const;

export type DammV2Form = (typeof DammV2Form)[keyof typeof DammV2Form];

export interface MeteoraDammV2Accounts {
  readonly pool: Address;
  readonly inputTokenAccount: Address;
  readonly outputTokenAccount: Address;
  readonly tokenAVault: Address;
  readonly tokenBVault: Address;
  readonly tokenAMint: Address;
  readonly tokenBMint: Address;
  /** The transaction signer whose token account is debited. */
  readonly payer: Address;
  readonly tokenAProgram: Address;
  readonly tokenBProgram: Address;
  /** The caller's referral fee token account, when the caller wants the referral share. */
  readonly referralTokenAccount: Address | undefined;
}

export function resolve(accounts: MeteoraDammV2Accounts, form: DammV2Form): VenueWindow {
  const fixed = [
    readonlyAccount(PROGRAM_ADDRESS),
    readonlyAccount(POOL_AUTHORITY),
    writableAccount(accounts.pool),
    writableAccount(accounts.inputTokenAccount),
    writableAccount(accounts.outputTokenAccount),
    writableAccount(accounts.tokenAVault),
    writableAccount(accounts.tokenBVault),
    readonlyAccount(accounts.tokenAMint),
    readonlyAccount(accounts.tokenBMint),
    readonlySignerAccount(accounts.payer),
    readonlyAccount(accounts.tokenAProgram),
    readonlyAccount(accounts.tokenBProgram),
    writableOrSentinel(accounts.referralTokenAccount, PROGRAM_ADDRESS),
    readonlyAccount(EVENT_AUTHORITY),
    readonlyAccount(PROGRAM_ADDRESS),
  ];
  switch (form) {
    case DammV2Form.Base:
      return new VenueWindow(HopKind.MeteoraDammV2, fixed);
    case DammV2Form.RateLimited:
      return new VenueWindow(HopKind.MeteoraDammV2, [
        ...fixed,
        readonlyAccount(INSTRUCTIONS_SYSVAR_ADDRESS),
      ]);
    default:
      return assertNever(form);
  }
}
