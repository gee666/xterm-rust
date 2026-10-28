/* tslint:disable */
/* eslint-disable */
/**
 * Handler for `console.log` invocations.
 *
 * If a test is currently running it takes the `args` array and stringifies
 * it and appends it to the current output of the test. Otherwise it passes
 * the arguments to the original `console.log` function, psased as
 * `original`.
 */
export function __wbgtest_console_log(args: Array<any>): void;
/**
 * Handler for `console.debug` invocations. See above.
 */
export function __wbgtest_console_debug(args: Array<any>): void;
/**
 * Handler for `console.info` invocations. See above.
 */
export function __wbgtest_console_info(args: Array<any>): void;
/**
 * Handler for `console.warn` invocations. See above.
 */
export function __wbgtest_console_warn(args: Array<any>): void;
/**
 * Handler for `console.error` invocations. See above.
 */
export function __wbgtest_console_error(args: Array<any>): void;
export function __wbgtest_cov_dump(): Uint8Array | undefined;
/**
 * Runtime test harness support instantiated in JS.
 *
 * The node.js entry script instantiates a `Context` here which is used to
 * drive test execution.
 */
export class WasmBindgenTestContext {
  free(): void;
  /**
   * Creates a new context ready to run tests.
   *
   * A `Context` is the main structure through which test execution is
   * coordinated, and this will collect output and results for all executed
   * tests.
   */
  constructor();
  /**
   * Handle `--include-ignored` flag.
   */
  include_ignored(include_ignored: boolean): void;
  /**
   * Handle filter argument.
   */
  filtered_count(filtered: number): void;
  /**
   * Executes a list of tests, returning a promise representing their
   * eventual completion.
   *
   * This is the main entry point for executing tests. All the tests passed
   * in are the JS `Function` object that was plucked off the
   * `WebAssembly.Instance` exports list.
   *
   * The promise returned resolves to either `true` if all tests passed or
   * `false` if at least one test failed.
   */
  run(tests: any[]): Promise<any>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbgt__wasm_options::inherited_input_order_and_unknown_getter_skipping: (a: number) => void;
  readonly __wbgt__wasm_options::inherited_object_prototype_names_are_real_constructor_keys: (a: number) => void;
  readonly __wbgt__wasm_options::dynamic_default_prototype_members_keep_full_string_key_events: (a: number) => void;
  readonly __wbgt__wasm_options::nested_default_bags_remain_live_without_root_assignment_events: (a: number) => void;
  readonly __wbgt__wasm_options::custom_prototype_inherited_accessors_stay_non_enumerable: (a: number) => void;
  readonly __wbgt__wasm_options::inherited_unknown_then_own_value_fails_public_spread: (a: number) => void;
  readonly __wbgt__wasm_options::null_and_primitive_prototype_assignment_change_membership: (a: number) => void;
  readonly __wbgt__wasm_options::prototype_getter_setter_receiver_and_post_assignment_value: (a: number) => void;
  readonly __wbgt__wasm_options::opaque_symbol_bigint_identity_truthiness_and_operators: (a: number) => void;
  readonly __wbgt__wasm_options::same_accessor_reentrancy_and_original_thrown_value_survive: (a: number) => void;
  readonly __wbgt__wasm_options::constructor_membership_proxy_throw_retains_identity: (a: number) => void;
  readonly __wbgt__wasm_options::default_hint_addition_and_concat_preserve_custom_primitive_order: (a: number) => void;
  readonly __wbgt__wasm_options::disposal_link_first_then_document_error_is_idempotent_and_lossless: (a: number) => void;
  readonly __wbgt__wasm_options::shared_default_objects_survive_shallow_copies_without_user_value_leaks: (a: number) => void;
  readonly __wbgt__wasm_options::public_descriptors_readonly_partial_assign_utf16_and_cleanup: (a: number) => void;
  readonly main: (a: number, b: number) => number;
  readonly __wbg_wasmbindgentestcontext_free: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_new: () => number;
  readonly wasmbindgentestcontext_include_ignored: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_filtered_count: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_run: (a: number, b: number, c: number) => any;
  readonly __wbgtest_console_log: (a: any) => void;
  readonly __wbgtest_console_debug: (a: any) => void;
  readonly __wbgtest_console_info: (a: any) => void;
  readonly __wbgtest_console_warn: (a: any) => void;
  readonly __wbgtest_console_error: (a: any) => void;
  readonly __wbgtest_cov_dump: () => [number, number];
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_1: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_export_5: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly closure25_externref_shim_multivalue_shim: (a: number, b: number, c: number, d: number, e: any) => [number, number];
  readonly __externref_table_dealloc: (a: number) => void;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7b2322c13905adaa_multivalue_shim: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly closure20_externref_shim: (a: number, b: number) => any;
  readonly closure261_externref_shim: (a: number, b: number, c: any) => void;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h89fd1e1ab6e3d13e_multivalue_shim: (a: number, b: number, c: number, d: number) => [number, number];
  readonly closure331_externref_shim_multivalue_shim: (a: number, b: number, c: number, d: number, e: any) => [number, number, number];
  readonly _dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd5c21b57d94ef8a5_multivalue_shim: (a: number, b: number) => [number, number, number];
  readonly closure335_externref_shim_multivalue_shim: (a: number, b: number, c: any) => [number, number];
  readonly wasm_bindgen__convert__closures__invoke0_mut__h6db07fd9af57efa3: (a: number, b: number) => void;
  readonly closure445_externref_shim: (a: number, b: number, c: any, d: number, e: any) => void;
  readonly closure437_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
