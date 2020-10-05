use anyhow::Result;

use crate::impls::{DFinanceDialect, LibraDialect};

use serde::export::fmt::Debug;

use std::str::FromStr;
use crate::shared::ProvidedAccountAddress;
use move_core_types::gas_schedule::CostTable;
use crate::shared::errors::FileSourceMap;

pub mod impls;
pub mod lang;
pub mod shared;
pub mod file;

pub trait Dialect {
    fn name(&self) -> &str;

    fn normalize_account_address(&self, addr: &str) -> Result<ProvidedAccountAddress>;

    fn cost_table(&self) -> CostTable;

    fn replace_addresses(&self, source_text: &str, source_map: &mut FileSourceMap) -> String;
}


#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DialectName {
    Libra,
    DFinance,
}

impl DialectName {
    pub fn get_dialect(&self) -> Box<dyn Dialect> {
        match self {
            DialectName::Libra => Box::new(LibraDialect::default()),
            DialectName::DFinance => Box::new(DFinanceDialect::default()),
        }
    }
}

impl FromStr for DialectName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "libra" => Ok(DialectName::Libra),
            "dfinance" => Ok(DialectName::DFinance),
            _ => Err(anyhow::format_err!("Invalid dialect {:?}", s)),
        }
    }
}
