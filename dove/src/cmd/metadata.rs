use anyhow::{Result, Error};
use structopt::StructOpt;
use crate::cmd::Cmd;
use crate::context::Context;

/// Metadata project command.
#[derive(StructOpt, Debug)]
pub struct Metadata {
    #[structopt(short = "j", long = "json")]
    json: bool,
}

impl Cmd for Metadata {
    fn apply(self, ctx: Context) -> Result<(), Error> {
        if self.json {
            println!("{}", serde_json::to_string_pretty(&ctx.manifest)?);
        } else {
            println!("{}", toml::to_string_pretty(&ctx.manifest)?);
        }

        Ok(())
    }
}
