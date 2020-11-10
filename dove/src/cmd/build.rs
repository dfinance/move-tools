use crate::cmd::Cmd;
use crate::context::Context;
use anyhow::Error;
use structopt::StructOpt;
use crate::index::Index;
use lang::compiler::file::{MoveFile, load_move_files};
use lang::builder::{Artifacts, MoveBuilder};
use termcolor::{StandardStream, ColorChoice};
use libra::compiler::output_errors;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::fs;
use libra::{prelude::CompiledUnit, compiler::*};

/// Build dependencies.
#[derive(StructOpt, Debug)]
pub struct Build {}

impl Cmd for Build {
    fn apply(self, ctx: Context) -> Result<(), Error> {
        let script_dir = ctx.path_for(&ctx.manifest.layout.script_dir);
        let module_dir = ctx.path_for(&ctx.manifest.layout.module_dir);

        let mut index = Index::load(&ctx)?;
        index.build()?;

        let dep_list = index.make_dependency_vec(&[&script_dir, &module_dir])?;

        let dep_list = dep_list
            .iter()
            .map(|path| path.as_str())
            .map(MoveFile::load)
            .collect::<Result<Vec<_>, Error>>()?;

        let source_list = load_move_files(&[script_dir, module_dir])?;

        let sender = ctx.account_address()?;
        let Artifacts { files, prog } =
            MoveBuilder::new(ctx.dialect.as_ref(), Some(sender).as_ref())
                .build(&source_list, &dep_list);

        match prog {
            Err(errors) => {
                let mut writer = StandardStream::stderr(ColorChoice::Auto);
                output_errors(&mut writer, files, errors);
                Err(anyhow!("could not compile:{}", ctx.project_name()))
            }
            Ok(compiled_units) => verify_and_store(&ctx, files, compiled_units),
        }
    }
}

pub fn verify_and_store(
    ctx: &Context,
    files: FilesSourceText,
    compiled_units: Vec<CompiledUnit>,
) -> Result<(), Error> {
    let (compiled_units, ice_errors) = compiled_unit::verify_units(compiled_units);
    let (modules, scripts): (Vec<_>, Vec<_>) = compiled_units
        .into_iter()
        .partition(|u| matches!(u, CompiledUnit::Module { .. }));

    fn store(units: Vec<CompiledUnit>, base_dir: &PathBuf) -> Result<(), Error> {
        for (idx, unit) in units.into_iter().enumerate() {
            let mut path = base_dir.join(format!("{}_{}", idx, unit.name()));
            path.set_extension("mv");
            File::create(&path)?.write_all(&unit.serialize())?
        }
        Ok(())
    }

    if !modules.is_empty() {
        let modules_dir = ctx.path_for(&ctx.manifest.layout.module_output);
        if modules_dir.exists() {
            fs::remove_dir_all(&modules_dir)?;
        }
        fs::create_dir_all(&modules_dir)?;

        store(modules, &modules_dir)?;
    }

    if !scripts.is_empty() {
        let scripts_dir = ctx.path_for(&ctx.manifest.layout.script_output);
        if scripts_dir.exists() {
            fs::remove_dir_all(&scripts_dir)?;
        }
        fs::create_dir_all(&scripts_dir)?;

        store(scripts, &scripts_dir)?;
    }

    if !ice_errors.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Auto);
        output_errors(&mut writer, files, ice_errors);
        Err(anyhow!("could not verify:{}", ctx.project_name()))
    } else {
        Ok(())
    }
}
