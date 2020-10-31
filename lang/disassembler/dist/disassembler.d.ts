/* tslint:disable */
/* eslint-disable */
/**
* @returns {any}
*/
export function version(): any;
/**
* @param {Uint8Array} bytes
* @param {boolean} compat_mode
* @returns {string | undefined}
*/
export function disassemble(bytes: Uint8Array, compat_mode: boolean): string | undefined;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly version: () => number;
  readonly disassemble: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
        