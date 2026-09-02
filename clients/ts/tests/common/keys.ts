import { getAddressDecoder, type Address } from "@solana/addresses";

/** A placeholder address of one repeated byte, the test suite's stand-in for a real key. */
export function key(byte: number): Address {
  return getAddressDecoder().decode(new Uint8Array(32).fill(byte));
}
