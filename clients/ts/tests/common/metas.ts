import { isSignerRole, isWritableRole, type AccountMeta } from "@solana/instructions";

import type { VenueWindow } from "../../src/index.js";

export function slot(window: VenueWindow, index: number): AccountMeta {
  const meta = window.accounts[index];
  if (meta === undefined) {
    throw new Error(`window has no slot ${String(index)}`);
  }
  return meta;
}

export function writable(meta: AccountMeta): boolean {
  return isWritableRole(meta.role);
}

export function signer(meta: AccountMeta): boolean {
  return isSignerRole(meta.role);
}
