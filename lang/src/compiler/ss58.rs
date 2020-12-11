use rust_base58::base58::FromBase58;
use hex;
use anyhow::{anyhow, ensure, Result};
use lazy_static::lazy_static;
use regex::Regex;
use blake2_rfc;
use crate::compiler::source_map::FileOffsetMap;

const SS58_PREFIX: &[u8]    = b"SS58PRE";
const PUB_KEY_LENGTH: usize = 32;

lazy_static! {
    static ref SS58_REGEX: Regex = Regex::new(
        r#"[1-9A-HJ-NP-Za-km-z]{40,}"#,
    )
    .unwrap();
}

fn ss58hash(data: &[u8]) -> blake2_rfc::blake2b::Blake2bResult {
	let mut context = blake2_rfc::blake2b::Blake2b::new(64);
	context.update(SS58_PREFIX);
	context.update(data);
	context.finalize()
}

pub fn ss58_to_libra(ss58: &str) -> Result<String> {
    let bs58 = match ss58.from_base58() {
        Ok(bs58) => bs58,
        Err(_) => return Err(anyhow!("Wrong base58"))
    };
    ensure!(bs58.len() == PUB_KEY_LENGTH+3, format!("Address length must be equal {} bytes", PUB_KEY_LENGTH+3));
    let mut addr = [0; 16];
    if bs58[PUB_KEY_LENGTH + 1..PUB_KEY_LENGTH + 3] != ss58hash(&bs58[0..PUB_KEY_LENGTH + 1]).as_bytes()[0..2] {
        return Err(anyhow!("Wrong address checksum"))
    }
    //addr[..2].copy_from_slice(&[bs58[0], 0]);
    addr.copy_from_slice(&bs58[1..PUB_KEY_LENGTH/2+1]);
    Ok(format!("0x{}", hex::encode_upper(addr).to_string()))
}

pub fn replace_ss58_addresses(source: &str, file_source_map: &mut FileOffsetMap) -> String {
    let mut transformed_source = source.to_string();

    for mat in SS58_REGEX.captures_iter(source).into_iter() {
        let item = mat.get(0).unwrap();

        let orig_address = item.as_str();
        if orig_address.starts_with("0x") {
            // libra match, don't replace
            continue;
        }
        if let Ok(libra_address) = ss58_to_libra(orig_address) {
            file_source_map.insert_address_layer(
                item.end(),
                orig_address.to_owned(),
                libra_address.clone(),
            );
            transformed_source = transformed_source.replace(orig_address, &libra_address);
        }
    }
    transformed_source
}
