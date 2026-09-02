import { getAddressEncoder, getProgramDerivedAddress, type Address } from "@solana/addresses";

import { ASSOCIATED_TOKEN_PROGRAM_ADDRESS } from "./programs.js";
import { CONFIG_SEED, ROUTER_PROGRAM_ADDRESS } from "./wire.js";

// Both derivations await crypto.subtle.digest inside getProgramDerivedAddress: local SHA-256 over
// the seeds, nothing sent anywhere. That is the only reason the builder is async.

/** The router's config account: `find_program_address([CONFIG_SEED], ROUTER_PROGRAM_ADDRESS)`. */
export async function findConfigAccountAddress(): Promise<Address> {
  const [configAccount] = await getProgramDerivedAddress({
    programAddress: ROUTER_PROGRAM_ADDRESS,
    seeds: [CONFIG_SEED],
  });
  return configAccount;
}

export async function findAssociatedTokenAddress(
  wallet: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<Address> {
  const encoder = getAddressEncoder();
  const [tokenAccount] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    seeds: [encoder.encode(wallet), encoder.encode(tokenProgram), encoder.encode(mint)],
  });
  return tokenAccount;
}
