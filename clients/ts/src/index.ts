/**
 * Builds the `find_route` instruction, and nothing else.
 *
 * The instruction takes three account sections in order: a six-slot prefix (the user, the base
 * token account, the base mint, the base token program, the router's config account and the fee
 * collector's token account), then one `(token program, user token account)` pair per route mint,
 * then the venue windows that make up the menu. `buildFindRouteInstruction` lays them out; the
 * `venues` modules build each window so that its declared account count cannot disagree with the
 * accounts it carries.
 *
 * Every wire number this package encodes lives in `wire`, and the test suite holds each one
 * against `wire/wire-manifest.json`. No other module carries a wire literal.
 *
 * This package discovers no pools, decodes no pool state, quotes no price, searches no cycle,
 * sizes no amount, derives none of the caller's token accounts, and builds no transaction. Those
 * are the caller's job or the program's.
 */
export {
  ALL_BASE_MINTS,
  BaseMint,
  buildFindRouteInstruction,
  findRouteFlagsToByte,
} from "./builder.js";
export type {
  FindRouteFlags,
  FindRouteInstruction,
  FindRouteParams,
  RouteMint,
} from "./builder.js";
export { FindRouteError, isFindRouteError } from "./error.js";
export type { FindRouteErrorDetail, FindRouteErrorKind } from "./error.js";
export { ALL_HOP_KINDS, HopKind, hopKindFromByte, hopKindName, isHopKind } from "./hopKind.js";
export type { HopKindName } from "./hopKind.js";
export { assertTailLength } from "./venues/tail.js";
export type { TupleOfLength, TupleRange } from "./venues/tail.js";
export type { VenueWindow } from "./venues/window.js";
export { WIRE_EPOCH } from "./wire.js";
export * as programs from "./programs.js";
export * as venues from "./venues/index.js";
export * as wire from "./wire.js";
