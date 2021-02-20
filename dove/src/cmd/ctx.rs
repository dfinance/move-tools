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
    move_lang::{compiled_unit, errors::output_errors},
};
use move_resource_viewer::tte::unwrap_spanned_ty;
use std::fmt::Debug;
use std::str::FromStr;
use lang::compiler::ss58::ss58_to_libra;
use std::fs;

/// Create transaction.
#[derive(StructOpt, Debug)]
pub struct CreateTransaction {
    #[structopt(help = "Script call declaration.\
     Example: 'create_balance<0x01::Dfinance::USD>([10,10], true, 68656c6c6f776f726c64, 100)'\
     ")]
    call: Option<String>,
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
            Err(anyhow!("Failed to determine script. There are several scripts. Use '--name' or '--file' to determine the script."))
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
            .filter(|(_, meta)| meta.iter().any(|meta| *name == meta.name))
            .collect::<Vec<_>>();

        if files.is_empty() {
            return Err(anyhow!("Script not found."));
        }

        if files.len() > 1 {
            let name_list = files
                .iter()
                .map(|(mf, _)| mf.name())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow!(
                "There are several scripts with the name '{:?}' in files ['{}'].",
                name,
                name_list
            ));
        }

        let (file, mut meta) = files.remove(0);
        if meta.is_empty() {
            return Err(anyhow!("Script not found."));
        }

        if meta.len() > 1 {
            return Err(anyhow!(
                "There are several scripts with the name '{:?}' in file '{}'.",
                name,
                file.name()
            ));
        }
        Ok((file, meta.remove(0)))
    }

    fn build_script(&self, ctx: &Context, script: MoveFile) -> Result<Vec<CompiledUnit>, Error> {
        let mut index = ctx.build_index()?;

        let module_dir = ctx
            .path_for(&ctx.manifest.layout.module_dir)
            .to_str()
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

    fn argument(&self, index: usize, total_expected: usize) -> Result<&String, Error> {
        self.args
            .get(index)
            .ok_or_else(|| anyhow!("{} arguments are expected.", total_expected))
    }

    fn prepare_arguments(
        &self,
        args_type: &[(String, String)],
    ) -> Result<(usize, usize, Vec<ScriptArg>), Error> {
        let total_args = args_type.len();

        fn parse_err<D: Debug>(name: &str, tp: &str, index: usize, value: &str, err: D) -> Error {
            anyhow!(
                "Parameter '{}' has {} type. Failed to parse {} [{}]. Error:'{:?}'",
                name,
                tp,
                value,
                index,
                err
            )
        }

        args_type.iter().try_fold(
            (0, 0, Vec::new()),
            |(signers, args_index, mut values), (name, tp)| match tp.as_str() {
                "&signer" => Ok((signers + 1, args_index, values)),
                "bool" => {
                    let arg = self.argument(args_index, total_args)?;
                    values.push(ScriptArg::Bool(
                        arg.parse()
                            .map_err(|err| parse_err(name, tp, args_index, arg, err))?,
                    ));
                    Ok((signers, args_index + 1, values))
                }
                "u8" => {
                    let arg = self.argument(args_index, total_args)?;
                    values.push(ScriptArg::U8(
                        arg.parse()
                            .map_err(|err| parse_err(name, tp, args_index, arg, err))?,
                    ));
                    Ok((signers, args_index + 1, values))
                }
                "u64" => {
                    let arg = self.argument(args_index, total_args)?;
                    values.push(ScriptArg::U64(
                        arg.parse()
                            .map_err(|err| parse_err(name, tp, args_index, arg, err))?,
                    ));
                    Ok((signers, args_index + 1, values))
                }
                "u128" => {
                    let arg = self.argument(args_index, total_args)?;
                    values.push(ScriptArg::U128(
                        arg.parse()
                            .map_err(|err| parse_err(name, tp, args_index, arg, err))?,
                    ));
                    Ok((signers, args_index + 1, values))
                }
                "address" => {
                    let arg = self.argument(args_index, total_args)?;
                    values.push(ScriptArg::Address(Address::from_str(arg)?.addr));
                    Ok((signers, args_index + 1, values))
                }
                "vector<u8>" => {
                    let arg = self.argument(args_index, total_args)?;
                    let buffer = if arg.contains('[') {
                        parse_vec(arg, "u8")?
                    } else {
                        hex::decode(arg)?
                    };
                    values.push(ScriptArg::VectorU8(buffer));
                    Ok((signers, args_index + 1, values))
                }
                "vector<u64>" => {
                    let arg = self.argument(args_index, total_args)?;
                    values.push(ScriptArg::VectorU64(parse_vec(arg, "u64")?));
                    Ok((signers, args_index + 1, values))
                }
                "vector<u128>" => {
                    let arg = self.argument(args_index, total_args)?;
                    values.push(ScriptArg::VectorU128(parse_vec(arg, "u128")?));
                    Ok((signers, args_index + 1, values))
                }
                "vector<address>" => {
                    let arg = self.argument(args_index, total_args)?;
                    let address = parse_vec::<Address>(arg, "vector<address>")?
                        .iter()
                        .map(|addr| addr.addr)
                        .collect();
                    values.push(ScriptArg::VectorAddress(address));
                    Ok((signers, args_index + 1, values))
                }
                &_ => Err(anyhow!("Unexpected script parameter: {}", tp)),
            },
        )
    }
}

impl Cmd for CreateTransaction {
    fn apply(self, ctx: Context) -> Result<(), Error> {
        let (script, meta) = self.lookup_script(&ctx)?;
        let units = self.build_script(&ctx, script)?;

        let unit = units
            .into_iter()
            .find(|unit| {
                let is_module = match &unit {
                    CompiledUnit::Module { .. } => false,
                    CompiledUnit::Script { .. } => true,
                };
                is_module && unit.name() == meta.name
            })
            .map(|unit| unit.serialize())
            .ok_or_else(|| anyhow!("Script '{}' not found", meta.name))?;

        if meta.type_parameters.len() != self.type_parameters.len() {
            return Err(anyhow!(
                "Script '{}' takes {} type parameters, {} passed",
                meta.name,
                meta.type_parameters.len(),
                self.type_parameters.len()
            ));
        }

        let type_parameters = self
            .type_parameters
            .iter()
            .map(|tp| parse_type_params(tp))
            .collect::<Result<Vec<_>, _>>()?;

        let (signers, args_count, args) = self.prepare_arguments(&meta.parameters)?;

        if self.args.len() != args_count {
            return Err(anyhow!(
                "Script '{}' takes {} parameters, {} passed",
                meta.name,
                args_count,
                self.args.len()
            ));
        }

        let tx = Transaction::new(signers as u8, unit, args, type_parameters);

        store_transaction(&ctx, &meta.name, tx)
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

impl Transaction {
    /// Create a new transaction.
    pub fn new(
        signers_count: u8,
        code: Vec<u8>,
        args: Vec<ScriptArg>,
        type_args: Vec<TypeTag>,
    ) -> Transaction {
        Transaction {
            signers_count,
            code,
            args,
            type_args,
        }
    }
}

fn parse_type_params(tkn: &str) -> Result<TypeTag, Error> {
    let map_err = |err| Error::msg(format!("{:?}", err));

    let mut lexer = Lexer::new(tkn, "query", Default::default());
    lexer.advance().map_err(map_err)?;

    let ty = parse_type(&mut lexer).map_err(map_err)?;
    unwrap_spanned_ty(ty)
}

fn parse_vec<E>(tkn: &str, tp_name: &str) -> Result<Vec<E>, Error>
where
    E: FromStr,
{
    let map_err = |err| Error::msg(format!("{:?}", err));

    let mut lexer = Lexer::new(tkn, "vec", Default::default());
    lexer.advance().map_err(map_err)?;

    if lexer.peek() != Tok::LBracket {
        return Err(anyhow!("Vector in format  [n1, n2, ..., nn] is expected."));
    }
    lexer.advance().map_err(map_err)?;

    let mut elements = vec![];
    while lexer.peek() != Tok::RBracket {
        match lexer.peek() {
            Tok::Comma => {
                lexer.advance().map_err(map_err)?;
                continue;
            }
            Tok::EOF => {
                return Err(anyhow!("unexpected end of vector."));
            }
            _ => {
                elements.push(E::from_str(lexer.content()).map_err(|_| {
                    anyhow!(
                        "Failed to parse vector element. {} type is expected. Actual:'{}'",
                        tp_name,
                        lexer.content()
                    )
                })?);
                lexer.advance().map_err(map_err)?;
            }
        }
    }
    Ok(elements)
}

fn store_transaction(ctx: &Context, name: &str, tx: Transaction) -> Result<(), Error> {
    let tx_dir = ctx.path_for(&ctx.manifest.layout.transaction_output);
    if !tx_dir.exists() {
        fs::create_dir_all(&tx_dir)?;
    }

    let mut tx_file = tx_dir.join(name);
    tx_file.set_extension("mvt");

    if tx_file.exists() {
        fs::remove_file(&tx_file)?;
    }
    println!("Store transaction:{:?}", tx_file);
    Ok(fs::write(&tx_file, libra::lcs::to_bytes(&tx)?)?)
}

struct Address {
    addr: AccountAddress,
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(addr: &str) -> Result<Self, Self::Err> {
        let addr = match ss58_to_libra(addr) {
            Ok(addr) => AccountAddress::from_hex_literal(&addr)?,
            Err(_) => AccountAddress::from_hex_literal(&addr)?,
        };
        Ok(Address { addr })
    }
}
