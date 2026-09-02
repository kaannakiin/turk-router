/**
 * One module per menu kind. Each exposes a `resolve` that takes the venue's accounts as named
 * fields — plus a tuple type for any variable tail — and returns a `VenueWindow` whose declared
 * account count is the length of what it holds. Slot 0 of every window is the venue's program.
 * Transfer-hook account groups are not built here.
 */
export * as meteoraDammV1 from "./meteoraDammV1.js";
export * as meteoraDammV2 from "./meteoraDammV2.js";
export * as meteoraDlmmSwap from "./meteoraDlmmSwap.js";
export * as meteoraDlmmSwap2 from "./meteoraDlmmSwap2.js";
export * as pumpSwapBuy from "./pumpSwapBuy.js";
export * as pumpSwapSell from "./pumpSwapSell.js";
export * as raydiumAmmV4 from "./raydiumAmmV4.js";
export * as raydiumClmm from "./raydiumClmm.js";
export * as raydiumCpmm from "./raydiumCpmm.js";
export * as whirlpool from "./whirlpool.js";
export { assertTailLength } from "./tail.js";
export type { TupleOfLength, TupleRange, TupleUpTo } from "./tail.js";
export type { VenueWindow } from "./window.js";
