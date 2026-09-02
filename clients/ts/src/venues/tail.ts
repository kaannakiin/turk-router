import type { Address } from "@solana/addresses";

import { FindRouteError } from "../error.js";

export type TupleOfLength<
  T,
  N extends number,
  Acc extends ReadonlyArray<T> = readonly [],
> = Acc["length"] extends N ? Acc : TupleOfLength<T, N, readonly [...Acc, T]>;

export type TupleUpTo<
  T,
  Max extends number,
  Acc extends ReadonlyArray<T> = readonly [],
> = Acc["length"] extends Max ? Acc : Acc | TupleUpTo<T, Max, readonly [...Acc, T]>;

/**
 * A venue's variable tail as a union of tuple types, `Min` to `Max` accounts long, so that a
 * literal outside the range is a compile error. A caller holding an array of unknown length
 * narrows it with {@link assertTailLength}.
 */
export type TupleRange<T, Min extends number, Max extends number> =
  Exclude<TupleUpTo<T, Max>, TupleUpTo<T, Min>> | TupleOfLength<T, Min>;

/** Throws `TailLength` unless `keys` holds `min..=max` accounts; every `resolve` calls it too. */
export function assertTailLength<Min extends number, Max extends number>(
  keys: ReadonlyArray<Address>,
  min: Min,
  max: Max,
): asserts keys is TupleRange<Address, Min, Max> {
  if (keys.length < min || keys.length > max) {
    throw new FindRouteError({ kind: "TailLength", given: keys.length, min, max });
  }
}
