/**
 * Raydium CPMM. The window is always 14 accounts:
 * `[0]` program (readonly), `[1]` user (signer), `[2]` swap authority (readonly),
 * `[3]` amm_config (readonly), `[4]` pool (writable), `[5]` input_token_account (writable),
 * `[6]` output_token_account (writable), `[7]` input_vault (writable), `[8]` output_vault
 * (writable), `[9]` input_token_program (readonly), `[10]` output_token_program (readonly),
 * `[11]` input_mint (readonly), `[12]` output_mint (readonly), `[13]` observation_state
 * (writable). Each token program independently names the Token program or the Token Extensions
 * program.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import { VenueWindow, readonlyAccount, readonlySignerAccount, writableAccount } from "./window.js";

/** The Raydium CPMM program. */
export const PROGRAM_ADDRESS: Address = address("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

/** The program-wide PDA every pool's swap instruction names as its authority. */
export const AUTHORITY: Address = address("GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL");

export interface RaydiumCpmmAccounts {
  /** The signer whose token accounts the swap moves through. */
  readonly user: Address;
  readonly ammConfig: Address;
  readonly pool: Address;
  readonly inputTokenAccount: Address;
  readonly outputTokenAccount: Address;
  readonly inputVault: Address;
  readonly outputVault: Address;
  /** The program that owns `inputTokenAccount` and `inputVault`. */
  readonly inputTokenProgram: Address;
  /** The program that owns `outputTokenAccount` and `outputVault`. */
  readonly outputTokenProgram: Address;
  readonly inputMint: Address;
  readonly outputMint: Address;
  readonly observationState: Address;
}

export function resolve(accounts: RaydiumCpmmAccounts): VenueWindow {
  return new VenueWindow(HopKind.RaydiumCpmm, [
    readonlyAccount(PROGRAM_ADDRESS),
    readonlySignerAccount(accounts.user),
    readonlyAccount(AUTHORITY),
    readonlyAccount(accounts.ammConfig),
    writableAccount(accounts.pool),
    writableAccount(accounts.inputTokenAccount),
    writableAccount(accounts.outputTokenAccount),
    writableAccount(accounts.inputVault),
    writableAccount(accounts.outputVault),
    readonlyAccount(accounts.inputTokenProgram),
    readonlyAccount(accounts.outputTokenProgram),
    readonlyAccount(accounts.inputMint),
    readonlyAccount(accounts.outputMint),
    writableAccount(accounts.observationState),
  ]);
}
