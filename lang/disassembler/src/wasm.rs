extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use crate::{Config, disasm_str};

const VERSION: &str = git_hash::crate_version_with_git_hash_short!();

#[wasm_bindgen]
pub extern "C" fn version() -> JsValue {
    JsValue::from_str(VERSION)
}

#[wasm_bindgen]
pub extern "C" fn disassemble(
    bytes: &[u8],
    compat_mode: bool,
) -> Result<Option<String>, JsValue> {
    let mut bytes = bytes.to_owned();

    if compat_mode {
        compat::adapt(&mut bytes).map_err(|err| err.to_string())?;
    }

    let cfg = Config {
        light_version: false,
    };

    let out = disasm_str(&bytes, cfg).map_err(|err| err.to_string())?;

    Ok(Some(out))
}
