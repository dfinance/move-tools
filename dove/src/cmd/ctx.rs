use crate::cmd::{Cmd, load_dependencies};
use crate::context::Context;
use anyhow::Error;
use structopt::StructOpt;
use lang::compiler::file::{MoveFile, find_move_files, load_move_files};
use lang::meta_extractor::{ScriptMetadata, Meta};
use lang::builder::{Artifacts, MoveBuilder};
use termcolor::{StandardStream, ColorChoice};
use libra::move_core_types::language_storage::TypeTag;
use serde::{Serialize, Deserialize};
use libra::account::AccountAddress;
use libra::move_lang::parser::lexer::{Lexer, Tok};
use libra::move_lang::parser::syntax::parse_type;
use libra::{
    prelude::CompiledUnit,
    move_lang::{
        compiled_unit,
        errors::output_errors,
    },
};
use move_resource_viewer::tte::unwrap_spanned_ty;

/// Create transaction.
#[derive(StructOpt, Debug)]
pub struct CreateTransaction {
    #[structopt(help = "Script name.", long = "name", short = "n")]
    script_name: Option<String>,
    #[structopt(help = "Script file name.", long = "file", short = "f")]
    file_name: Option<String>,
    #[structopt(
    help = r#"Script type parametrs, e.g. 0x1::Dfinance::USD"#,
    name = "Script type parameters.",
    long = "type",
    short = "t"
    )]
    type_parameters: Vec<String>,
    #[structopt(
    help = r#"Script arguments, e.g. 10 20 30"#,
    name = "Script arguments.",
    long = "args",
    short = "a"
    )]
    args: Vec<String>,
}

impl CreateTransaction {
    fn lookup_script(&self, ctx: &Context) -> Result<(MoveFile, Meta), Error> {
        if let Some(file_name) = &self.file_name {
            return self.lookup_script_by_file_name(ctx, file_name);
        }

        if let Some(name) = &self.script_name {
            return self.lookup_script_by_name(ctx, name);
        }

        let script_path = ctx.path_for(&ctx.manifest.layout.script_dir);
        let files = find_move_files(&script_path)?;
        if files.len() == 1 {
            let mf = MoveFile::load(&files[0])?;
            let mut meta = ScriptMetadata::extract(ctx.dialect.as_ref(), &mf)?;
            if meta.is_empty() {
                return Err(anyhow!("Script not found."));
            }
            if meta.len() > 1 {
                return Err(anyhow!("Failed to determine script. There are several scripts. Use '--name' to determine the script."));
            }
            Ok((mf, meta.remove(0)))
        } else {
            return Err(anyhow!("Failed to determine script. There are several scripts. Use '--name' or '--file' to determine the script."));
        }
    }

    fn lookup_script_by_file_name(
        &self,
        ctx: &Context,
        fname: &str,
    ) -> Result<(MoveFile, Meta), Error> {
        let script_path = ctx.path_for(&ctx.manifest.layout.script_dir);
        let file_path = if !fname.ends_with("move") {
            let mut path = script_path.join(fname);
            path.set_extension("move");
            path
        } else {
            script_path.join(fname)
        };
        if !file_path.exists() {
            return Err(anyhow!("File [{}] not found", fname));
        }

        let script = MoveFile::load(&file_path)?;
        let mut scripts = ScriptMetadata::extract(ctx.dialect.as_ref(), &script)?;
        if scripts.is_empty() {
            return Err(anyhow!("Script not found in file '{}'", fname));
        }

        let meta = if scripts.len() > 1 {
            let mut scripts = scripts
                .into_iter()
                .filter(|sc| {
                    if let Some(script_name) = &self.script_name {
                        &sc.name == script_name
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();
            if scripts.len() > 1 {
                return Err(anyhow!(
                    "There are several scripts with the name '{:?}' in file '{}'",
                    self.script_name,
                    fname
                ));
            } else {
                scripts.remove(0)
            }
        } else {
            scripts.remove(0)
        };

        Ok((script, meta))
    }

    fn lookup_script_by_name(
        &self,
        ctx: &Context,
        name: &str,
    ) -> Result<(MoveFile, Meta), Error> {
        let script_path = ctx.path_for(&ctx.manifest.layout.script_dir);
        let mut files = find_move_files(&script_path)?
            .iter()
            .map(MoveFile::load)
            .filter_map(|mf| match mf {
                Ok(mf) => {
                    if mf.content().contains(name) {
                        Some(mf)
                    } else {
                        None
                    }
                }
                Err(err) => {
                    warn!("{:?}", err);
                    None
                }
            })
            .map(|mf| ScriptMetadata::extract(ctx.dialect.as_ref(), &mf).map(|meta| (mf, meta)))
            .filter_map(|script| match script {
                Ok((mf, meta)) => Some((mf, meta)),
                Err(err) => {
                    warn!("{:?}", err);
                    None
                }
            })
            .filter(|(_, meta)| meta.iter().any(|meta| &meta.name == name))
            .collect::<Vec<_>>();

        if files.is_empty() {
            return Err(anyhow!("Script not found."));
        }

        if files.len() > 1 {
            let name_list = files.iter().map(|(mf, _)| mf.name()).collect::<Vec<_>>().join(", ");
            return Err(anyhow!("There are several scripts with the name '{:?}' in files ['{}'].", name, name_list));
        }

        let (file, mut meta) = files.remove(0);
        if meta.is_empty() {
            return Err(anyhow!("Script not found."));
        }

        if meta.len() > 1 {
            return Err(anyhow!("There are several scripts with the name '{:?}' in file '{}'.", name, file.name()));
        }
        Ok((file, meta.remove(0)))
    }

    fn build_script(&self, ctx: &Context, script: MoveFile) -> Result<Vec<CompiledUnit>, Error> {
        let mut index = ctx.build_index()?;

        let module_dir = ctx.path_for(&ctx.manifest.layout.module_dir).to_str()
            .map(|path| path.to_owned())
            .ok_or_else(|| anyhow!("Failed to convert module dir path"))?;

        let dep_set = index.make_dependency_set(&[module_dir.as_str(), script.name()])?;
        let mut dep_list = load_dependencies(dep_set)?;
        dep_list.extend(load_move_files(&[module_dir])?);

        let sender = ctx.account_address()?;
        let Artifacts { files, prog } =
            MoveBuilder::new(ctx.dialect.as_ref(), Some(sender).as_ref())
                .build(&[script], &dep_list);

        match prog {
            Err(errors) => {
                let mut writer = StandardStream::stderr(ColorChoice::Auto);
                output_errors(&mut writer, files, errors);
                Err(anyhow!("could not compile:{}", ctx.project_name()))
            }
            Ok(compiled_units) => {
                let (compiled_units, ice_errors) = compiled_unit::verify_units(compiled_units);

                if !ice_errors.is_empty() {
                    let mut writer = StandardStream::stderr(ColorChoice::Auto);
                    output_errors(&mut writer, files, ice_errors);
                    Err(anyhow!("could not verify:{}", ctx.project_name()))
                } else {
                    Ok(compiled_units)
                }
            }
        }
    }

    fn prepare_arguments(&self, args_type: &Vec<(String, String)>) -> Result<(usize, usize, Vec<ScriptArg>), Error> {
        args_type.iter()
            .map(|(_, tp)| tp)
            .try_fold((0, 0, Vec::new()), |(signers, args_index, mut values), tp| {
                match tp.as_str() {
                    "&signer" => {
                        Ok((signers + 1, args_index, values))
                    }
                    "bool" => {
                        self.type_parameters.get(args_index)
                            .ok_or_else(|| anyhow!("{} arguments are expected.", args_type.len()))?;
                        Ok((signers, args_index+ 1, values))
                    }
                    &_ => {
                        Err(anyhow!("Unexpected script parameter: {}", tp))
                    }
                }
            })
    }
}

impl Cmd for CreateTransaction {
    fn apply(self, ctx: Context) -> Result<(), Error> {
        let (script, meta) = self.lookup_script(&ctx)?;
        let units = self.build_script(&ctx, script)?;

        let unit = units.into_iter()
            .find(|unit| {
                let is_module = match &unit {
                    CompiledUnit::Module { .. } => false,
                    CompiledUnit::Script { .. } => true
                };
                is_module && unit.name() == meta.name
            })
            .map(|unit| unit.serialize())
            .ok_or_else(|| anyhow!("Script '{}' not found", meta.name))?;

        if meta.type_parameters.len() != self.type_parameters.len() {
            return Err(anyhow!("Script '{}' takes {} type parameters, {} passed", meta.name, meta.type_parameters.len(), self.type_parameters.len()));
        }

        let type_parameters = self.type_parameters.iter()
            .map(|tp| parse_type_params(tp))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}

/// Script argument type.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ScriptArg {
    /// u8
    U8(u8),
    /// u64
    U64(u64),
    /// u128
    U128(u128),
    /// bool
    Bool(bool),
    /// address
    Address(AccountAddress),
    /// vector<u8>
    VectorU8(Vec<u8>),
    /// vector<u64>
    VectorU64(Vec<u64>),
    /// vector<u128>
    VectorU128(Vec<u128>),
    /// vector<bool>
    VectorBool(Vec<bool>),
    /// vector<address>
    VectorAddress(Vec<AccountAddress>),
}

/// Transaction model.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    signers_count: u8,
    code: Vec<u8>,
    args: Vec<ScriptArg>,
    type_args: Vec<TypeTag>,
}

fn parse_type_params(tkn: &str) -> Result<TypeTag, Error> {
    let map_err = |err| Error::msg(format!("{:?}", err));

    let mut lexer = Lexer::new(tkn, "query", Default::default());
    lexer.advance().map_err(map_err)?;

    while lexer.peek() != Tok::EOF {
        let ty = parse_type(&mut lexer).map_err(map_err)?;
        return Ok(unwrap_spanned_ty(ty)?);
    }
    Err(anyhow!("Failed to parse type parameter:{}", tkn))
}

