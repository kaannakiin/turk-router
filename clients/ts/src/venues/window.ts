import type { Address } from "@solana/addresses";
import { AccountRole, type AccountMeta } from "@solana/instructions";

import type { HopKind } from "../hopKind.js";
import type { MenuEntry } from "../wire.js";

/**
 * One pool's accounts, laid out the way its venue's swap instruction takes them. Only the venue
 * modules construct one; the count the menu entry declares is the length of what it holds, so the
 * two cannot disagree. The private fields make the type nominal: an object literal cannot pass as
 * a window, and a window does not survive `structuredClone` or JSON — rebuild it instead.
 */
export class VenueWindow {
  readonly #hopKind: HopKind;
  readonly #accounts: ReadonlyArray<AccountMeta>;

  constructor(hopKind: HopKind, accounts: ReadonlyArray<AccountMeta>) {
    this.#hopKind = hopKind;
    this.#accounts = Object.freeze([...accounts]);
  }

  /** The venue this window is for. */
  get hopKind(): HopKind {
    return this.#hopKind;
  }

  /** What the menu entry declares: the number of accounts the window carries, slot 0 included. */
  get accountCount(): number {
    return this.#accounts.length;
  }

  /** The accounts, slot 0 first. */
  get accounts(): ReadonlyArray<AccountMeta> {
    return this.#accounts;
  }

  /** @internal */
  menuEntry(): MenuEntry {
    return {
      hopKind: this.#hopKind,
      accountCount: this.#accounts.length,
      hookLen0: 0,
      hookLen1: 0,
    };
  }
}

export function readonlyAccount(address: Address): AccountMeta {
  return { address, role: AccountRole.READONLY };
}

export function writableAccount(address: Address): AccountMeta {
  return { address, role: AccountRole.WRITABLE };
}

export function readonlySignerAccount(address: Address): AccountMeta {
  return { address, role: AccountRole.READONLY_SIGNER };
}

export function writableSignerAccount(address: Address): AccountMeta {
  return { address, role: AccountRole.WRITABLE_SIGNER };
}

/** An optional account the venue receives a transfer into: writable when given, else the sentinel. */
export function writableOrSentinel(account: Address | undefined, sentinel: Address): AccountMeta {
  return account === undefined ? readonlyAccount(sentinel) : writableAccount(account);
}
