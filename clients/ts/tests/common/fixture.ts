/**
 * The fixture text format, parsed and rendered byte for byte — a port of the Rust test suite's
 * `tests/common/fixture.rs`, with the same grammar so a malformed file fails the same way on
 * both sides.
 */
import { getAddressDecoder, getAddressEncoder, type Address } from "@solana/addresses";
import { getBase16Decoder, getBase16Encoder } from "@solana/codecs-strings";

export type Role = "fixed" | "in_ata" | "out_ata" | "payer" | "program_ref";

const ROLES: ReadonlyArray<Role> = ["fixed", "in_ata", "out_ata", "payer", "program_ref"];

export interface Body {
  readonly owner: Address;
  readonly lamports: bigint;
  readonly data: Uint8Array;
}

export interface Slot {
  readonly writable: boolean;
  readonly signer: boolean;
  readonly role: Role;
  readonly pubkey: Address;
  readonly body: Body | undefined;
}

export interface Extra {
  readonly role: string;
  readonly pubkey: Address;
  readonly body: Body;
}

export interface Fixture {
  readonly kind: string;
  readonly poolB58: string;
  readonly slot: bigint;
  readonly unixTs: bigint;
  readonly hopKind: number;
  readonly programId: Address;
  readonly inputMint: Address;
  readonly outputMint: Address;
  readonly inputTokenProgram: Address;
  readonly outputTokenProgram: Address;
  readonly hookLens: readonly [number, number];
  readonly slots: Array<Slot>;
  readonly extras: Array<Extra>;
}

export function windowPubkeys(fixture: Fixture): Array<Address> {
  return [fixture.programId, ...fixture.slots.map((slot) => slot.pubkey)];
}

export function slotAt(fixture: Fixture, index: number): Slot {
  const slot = fixture.slots[index];
  if (slot === undefined) {
    throw new Error(`fixture has no slot ${index}`);
  }
  return slot;
}

interface PendingSlot {
  readonly kind: "slot";
  readonly writable: boolean;
  readonly signer: boolean;
  readonly role: Role;
}

interface PendingExtra {
  readonly kind: "extra";
  readonly role: string;
}

interface Pending {
  header: PendingSlot | PendingExtra | undefined;
  pubkey: Address | undefined;
  owner: Address | undefined;
  lamports: bigint | undefined;
  data: Uint8Array | undefined;
}

export function parse(text: string): Fixture {
  const header: Record<string, string> = {};
  let hookLens: readonly [number, number] = [0, 0];
  const slots: Array<Slot> = [];
  const extras: Array<Extra> = [];
  const pending: Pending = {
    header: undefined,
    pubkey: undefined,
    owner: undefined,
    lamports: undefined,
    data: undefined,
  };

  const flush = (): void => {
    const block = pending.header;
    if (block === undefined) {
      return;
    }
    if (pending.pubkey === undefined) {
      throw new Error("account block has no pubkey");
    }
    const bodyFields = [pending.owner, pending.lamports, pending.data];
    const present = bodyFields.filter((value) => value !== undefined).length;
    let body: Body | undefined;
    if (present === 3 && pending.owner !== undefined && pending.lamports !== undefined) {
      body = {
        owner: pending.owner,
        lamports: pending.lamports,
        data: pending.data ?? new Uint8Array(),
      };
    } else if (present !== 0) {
      throw new Error("partial account body");
    }
    if (block.kind === "slot") {
      slots.push({
        writable: block.writable,
        signer: block.signer,
        role: block.role,
        pubkey: pending.pubkey,
        body,
      });
    } else {
      if (body === undefined) {
        throw new Error("extra has no body");
      }
      extras.push({ role: block.role, pubkey: pending.pubkey, body });
    }
    pending.header = undefined;
    pending.pubkey = undefined;
    pending.owner = undefined;
    pending.lamports = undefined;
    pending.data = undefined;
  };

  for (const rawLine of text.split("\n")) {
    const line = rawLine.endsWith("\r") ? rawLine.slice(0, -1) : rawLine;
    if (line.length === 0) {
      continue;
    }
    const colon = line.indexOf(":");
    const key = colon === -1 ? line : line.slice(0, colon).trim();
    const value = colon === -1 ? "" : line.slice(colon + 1).trim();
    switch (key) {
      case "account":
        flush();
        pending.header = {
          kind: "slot",
          writable: headerField(value, "writable=") === "true",
          signer: headerField(value, "signer=") === "true",
          role: parseRole(headerField(value, "role=")),
        };
        break;
      case "extra":
        flush();
        pending.header = { kind: "extra", role: headerField(value, "role=") };
        break;
      case "pubkey":
        pending.pubkey = parseKey(value);
        break;
      case "owner":
        pending.owner = parseKey(value);
        break;
      case "lamports":
        pending.lamports = BigInt(value);
        break;
      case "data_hex":
        pending.data = hexDecode(value);
        break;
      case "hook_lens": {
        const [first, second] = value.split(",");
        if (first === undefined || second === undefined) {
          throw new Error("hook_lens is 'A,B'");
        }
        hookLens = [Number(first.trim()), Number(second.trim())];
        break;
      }
      case "kind":
      case "pool_b58":
      case "slot":
      case "unix_ts":
      case "hop_kind":
      case "program_id":
      case "input_mint":
      case "output_mint":
      case "input_token_program":
      case "output_token_program":
        header[key] = value;
        break;
      default:
        break;
    }
  }
  flush();

  return {
    kind: headerText(header, "kind"),
    poolB58: headerText(header, "pool_b58"),
    slot: BigInt(headerText(header, "slot")),
    unixTs: BigInt(headerText(header, "unix_ts")),
    hopKind: Number(headerText(header, "hop_kind")),
    programId: parseKey(headerText(header, "program_id")),
    inputMint: parseKey(headerText(header, "input_mint")),
    outputMint: parseKey(headerText(header, "output_mint")),
    inputTokenProgram: parseKey(headerText(header, "input_token_program")),
    outputTokenProgram: parseKey(headerText(header, "output_token_program")),
    hookLens,
    slots,
    extras,
  };
}

export function render(fixture: Fixture): string {
  let out = "";
  out += `kind: ${fixture.kind}\n`;
  out += `pool_b58: ${fixture.poolB58}\n`;
  out += `slot: ${String(fixture.slot)}\n`;
  out += `unix_ts: ${String(fixture.unixTs)}\n`;
  out += `hop_kind: ${String(fixture.hopKind)}\n`;
  out += `program_id: ${formatKey(fixture.programId)}\n`;
  out += `input_mint: ${formatKey(fixture.inputMint)}\n`;
  out += `output_mint: ${formatKey(fixture.outputMint)}\n`;
  out += `input_token_program: ${formatKey(fixture.inputTokenProgram)}\n`;
  out += `output_token_program: ${formatKey(fixture.outputTokenProgram)}\n`;
  out += `accounts: ${String(fixture.slots.length)}\n`;
  if (fixture.hookLens[0] !== 0 || fixture.hookLens[1] !== 0) {
    out += `hook_lens: ${String(fixture.hookLens[0])},${String(fixture.hookLens[1])}\n`;
  }
  fixture.slots.forEach((slot, index) => {
    out += `account: index=${String(index)} writable=${String(slot.writable)} signer=${String(slot.signer)} role=${slot.role}\n`;
    out += `pubkey: ${formatKey(slot.pubkey)}\n`;
    if (slot.body !== undefined) {
      out += renderBody(slot.body);
    }
  });
  for (const extra of fixture.extras) {
    out += `extra: role=${extra.role}\n`;
    out += `pubkey: ${formatKey(extra.pubkey)}\n`;
    out += renderBody(extra.body);
  }
  return out;
}

function renderBody(body: Body): string {
  return (
    `owner: ${formatKey(body.owner)}\n` +
    `lamports: ${String(body.lamports)}\n` +
    `data_hex: ${getBase16Decoder().decode(body.data)}\n`
  );
}

function headerText(header: Record<string, string>, key: string): string {
  const value = header[key];
  if (value === undefined) {
    throw new Error(`${key} header`);
  }
  return value;
}

function headerField(header: string, key: string): string {
  const from = header.indexOf(key);
  if (from === -1) {
    throw new Error(`header field ${key}`);
  }
  const rest = header.slice(from + key.length);
  const token = rest.split(/\s+/)[0];
  if (token === undefined || token.length === 0) {
    throw new Error(`header field value ${key}`);
  }
  return token;
}

function parseRole(text: string): Role {
  const role = ROLES.find((candidate) => candidate === text);
  if (role === undefined) {
    throw new Error(`unknown role ${text}`);
  }
  return role;
}

function parseKey(value: string): Address {
  if (!value.startsWith("[") || !value.endsWith("]")) {
    throw new Error("a bracketed key");
  }
  const bytes = value
    .slice(1, -1)
    .split(",")
    .map((token) => Number(token.trim()));
  if (
    bytes.length !== 32 ||
    bytes.some((byte) => !Number.isInteger(byte) || byte < 0 || byte > 255)
  ) {
    throw new Error("32 bytes");
  }
  return getAddressDecoder().decode(new Uint8Array(bytes));
}

function formatKey(address: Address): string {
  return `[${Array.from(getAddressEncoder().encode(address), (byte) => String(byte)).join(", ")}]`;
}

// Kit's base16 encoder silently truncates an odd-length string; the Rust parser asserts, so this
// one does too.
function hexDecode(text: string): Uint8Array {
  if (text.length % 2 !== 0) {
    throw new Error("odd-length hex");
  }
  return new Uint8Array(getBase16Encoder().encode(text));
}
