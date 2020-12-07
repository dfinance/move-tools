// Example use:
// API: "https://rest.demo1.dfinance.co"
// Simple queries:
// "0x1::Account::Balance<0x1::XFI::T>",
// "0x1::Account::Balance<0x1::Coins::ETH>",
// "0x1::Account::Balance<0x1::Coins::BTC>",
// "0x1::Account::Balance<0x1::Coins::USDT>",
// "0x1::Account::Balance<0x1::Coins::SXFI>",

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

use std::path::{Path, PathBuf};
use anyhow::{Result, Error, anyhow};
use http::Uri;
use clap::Clap;
use libra::prelude::*;
use lang::compiler::bech32::{bech32_into_libra, HRP};

mod net;
mod ser;
mod tte;

pub const VERSION: &str = git_hash::crate_version_with_git_hash_short!();

#[derive(Clap, Debug)]
#[clap(name = "Move resource viewer", version = VERSION)]
struct Cfg {
    /// Owner's address
    #[clap(long, short)]
    address: String,

    /// Query in `TypeTag` format.
    /// Mainly, in most cases should be StructTag.
    /// Additionaly can contain index at the end.
    /// Query examples:
    /// "0x1::Account::Balance<0x1::XFI::T>",
    /// "0x1::Account::Balance<0x1::Coins::ETH>"
    #[clap(long, short)]
    query: tte::TypeTagExt,

    /// Output file path
    #[clap(long, short)]
    output: PathBuf,

    /// Sets output format to JSON.
    /// Optional, `true` if output file extension is .json
    #[clap(long, short)]
    json: Option<bool>,

    /// Node REST API address
    #[clap(long)]
    api: Uri,

    /// Enables compatibility mode
    #[clap(long, short)]
    compat: bool,
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err)
    } else {
        info!("completed successfully")
    }
}

fn run() -> Result<(), Error> {
    env_logger::init();

    let cfg = Cfg::parse();
    let host = cfg.api;
    let output = cfg.output;
    let json = cfg.json.unwrap_or_else(|| {
        output
            .extension()
            .map(|ext| ext == "json")
            .unwrap_or_default()
    });
    let (tte, index) = cfg.query.into_inner();
    let addr = if cfg.address.starts_with(HRP) {
        AccountAddress::from_hex_literal(&bech32_into_libra(&cfg.address)?)
    } else {
        AccountAddress::from_hex_literal(&cfg.address)
    }?;

    match tte {
        TypeTag::Struct(st) => {
            let key = ResourceKey::new(addr, st.clone());
            let res = net::get_resource(&key, &host);
            res.map(|bytes| {
                if bytes.len() > 0 {
                    let client = net::DnodeRestClient::new(host);

                    // Internally produce FatStructType (with layout) for StructTag by
                    // resolving & de-.. entire deps-chain.
                    let annotator = rv::MoveValueAnnotator::new_no_stdlib(&client);

                    annotator
                        .view_resource(&st, &bytes)
                        .and_then(|result| {
                            // debug!("result: {:#?}", result);

                            if json {
                                serde_json::ser::to_string_pretty(
                                    &ser::AnnotatedMoveStructHelper(result),
                                )
                                .map_err(|err| anyhow!("{}", err))
                            } else {
                                Ok(format!("{}", result))
                            }
                        })
                        .map(|result| write_output(&output, result))
                } else {
                    Err(anyhow!("Err: res is empty"))
                }
            })
            .and_then(|result| result)
        }

        TypeTag::Vector(tt) => {
            // TODO: query using index, seed
            Err(anyhow!(
                "Err: unsupported root type Vec<{}>{:?}",
                tt,
                index.map(|v| [v]).unwrap_or(Default::default())
            ))
        }

        _ => Err(anyhow!("Err: unsupported type {}", tte)),
    }
}

fn write_output(path: &Path, result: String) {
    use std::io::prelude::*;
    std::fs::File::create(path)
        .and_then(|mut f| f.write_all(result.as_bytes()))
        .map_err(|err| error!("Err: cannot write output: {}", err))
        .ok();
}
