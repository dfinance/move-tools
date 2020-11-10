use crate::cmd::Cmd;
use crate::context::Context;
use anyhow::Error;
use structopt::StructOpt;

/// Run script.
#[derive(StructOpt, Debug)]
pub struct Run {}

impl Cmd for Run {
    fn apply(self, _ctx: Context) -> Result<(), Error> {
        todo!()
    }
}
