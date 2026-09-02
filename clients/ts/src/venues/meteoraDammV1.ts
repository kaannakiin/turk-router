/**
 * Meteora DAMM v1 (Dynamic AMM), `swap`. The window is always 16 accounts:
 * `[0]` program (readonly), `[1]` pool (writable), `[2]` user_source (writable),
 * `[3]` user_dest (writable), `[4]` a_vault (writable), `[5]` b_vault (writable),
 * `[6]` a_token_vault (writable), `[7]` b_token_vault (writable), `[8]` a_vault_lp_mint
 * (writable), `[9]` b_vault_lp_mint (writable), `[10]` a_vault_lp (writable), `[11]` b_vault_lp
 * (writable), `[12]` protocol_token_fee (writable), `[13]` payer (signer), `[14]` Dynamic Vault
 * program (readonly), `[15]` Token program (readonly). The classic Token program only.
 */
import { address, type Address } from "@solana/addresses";

import { HopKind } from "../hopKind.js";
import { TOKEN_PROGRAM_ADDRESS } from "../programs.js";
import { VenueWindow, readonlyAccount, readonlySignerAccount, writableAccount } from "./window.js";

/** The Meteora DAMM v1 program. */
export const PROGRAM_ADDRESS: Address = address("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");

/** The Meteora Dynamic Vault program every DAMM v1 swap names, one slot after the payer. */
export const DYNAMIC_VAULT_PROGRAM_ADDRESS: Address = address(
  "24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi",
);

export interface MeteoraDammV1Accounts {
  readonly pool: Address;
  /** The caller's token account this hop debits. */
  readonly userSource: Address;
  /** The caller's token account this hop credits. */
  readonly userDest: Address;
  readonly aVault: Address;
  readonly bVault: Address;
  readonly aTokenVault: Address;
  readonly bTokenVault: Address;
  readonly aVaultLpMint: Address;
  readonly bVaultLpMint: Address;
  readonly aVaultLp: Address;
  readonly bVaultLp: Address;
  /** The pool's protocol fee token account for the side being sold. */
  readonly protocolTokenFee: Address;
  /** The wallet authorizing the debit from `userSource`. */
  readonly payer: Address;
}

export function resolve(accounts: MeteoraDammV1Accounts): VenueWindow {
  return new VenueWindow(HopKind.MeteoraDammV1, [
    readonlyAccount(PROGRAM_ADDRESS),
    writableAccount(accounts.pool),
    writableAccount(accounts.userSource),
    writableAccount(accounts.userDest),
    writableAccount(accounts.aVault),
    writableAccount(accounts.bVault),
    writableAccount(accounts.aTokenVault),
    writableAccount(accounts.bTokenVault),
    writableAccount(accounts.aVaultLpMint),
    writableAccount(accounts.bVaultLpMint),
    writableAccount(accounts.aVaultLp),
    writableAccount(accounts.bVaultLp),
    writableAccount(accounts.protocolTokenFee),
    readonlySignerAccount(accounts.payer),
    readonlyAccount(DYNAMIC_VAULT_PROGRAM_ADDRESS),
    readonlyAccount(TOKEN_PROGRAM_ADDRESS),
  ]);
}
