use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use clap::{App, Arg};
use move_executor::compile_and_run_scripts_in_file;
use move_executor::explain::{PipelineExecutionResult, StepExecutionResult};
use utils::{leaked_fpath, MoveFilePath};
use lang::compiler::errors::{ExecCompilerError, report_errors};
use move_lang::errors::FilesSourceText;
use lang::file;
use lang::file::MvFile;
use move_lang::name_pool::ConstPool;

fn get_files_for_error_reporting(script: MvFile, deps: Vec<MvFile>) -> FilesSourceText {
    // let mut mapping = FilesSourceText::with_capacity(deps.len() + 1);
    // for (fpath, text) in vec![script].into_iter().chain(deps.into_iter()) {
    //     mapping.insert(fpath, text);
    // }
    // mapping
    todo!()
}

fn main() -> Result<()> {
    let cli_arguments = App::new("Move Executor")
        .version(git_hash::crate_version_with_git_hash_short!())
        .arg(
            Arg::with_name("SCRIPT")
                .required(true)
                .help("Path to script to execute"),
        )
        .arg(
            Arg::from_usage("-d --dialect=[DIALECT]")
                .possible_values(&["libra", "dfinance"])
                .default_value("libra")
                .help("Move language dialect"),
        )
        .arg(
            Arg::from_usage("-s --sender [SENDER_ADDRESS]")
                .required(true)
                .help("Address of the current user"),
        )
        .arg(
            Arg::from_usage("-m --modules [MODULE_PATHS]")
                .multiple(true)
                .number_of_values(1)
                .help("Path to module file / modules folder to use as dependency. \nCould be used more than once: '-m ./stdlib -m ./modules'"),
        )
        .arg(Arg::from_usage("--show-changes").help("Show what changes has been made to the network after script is executed"))
        .arg(Arg::from_usage("--show-events").help("Show which events was emitted"))
        .arg(
            Arg::from_usage("--args [SCRIPT_ARGS]")
                .help(r#"Number of script main() function arguments in quotes, e.g. "10 20 30""#),
        )
        .get_matches();

    let _pool = ConstPool::new();
    let script = MvFile::with_path(cli_arguments.value_of("SCRIPT").unwrap().to_owned())?;
    let modules_fpaths = cli_arguments
        .values_of("modules")
        .unwrap_or_default()
        .map(|path| path.into())
        .collect::<Vec<PathBuf>>();
    let deps = file::load_move_files(modules_fpaths.as_slice())?;

    let show_network_effects = cli_arguments.is_present("show-changes");
    let show_events = cli_arguments.is_present("show-events");

    let dialect = cli_arguments.value_of("dialect").unwrap();
    let sender = cli_arguments.value_of("sender").unwrap();
    let args: Vec<String> = cli_arguments
        .value_of("args")
        .unwrap_or_default()
        .split_ascii_whitespace()
        .map(String::from)
        .collect();

    let res = compile_and_run_scripts_in_file(&script, &deps, dialect, sender, args);

    match res {
        Ok(exec_result) => {
            let PipelineExecutionResult {
                gas_spent,
                step_results,
            } = exec_result;
            println!("Gas used: {}", gas_spent);

            for (name, step_result) in step_results {
                println!("{}: ", name);
                match step_result {
                    StepExecutionResult::Error(error) => {
                        print!("{}", textwrap::indent(&error, "    "));
                    }
                    StepExecutionResult::Success(effects) => {
                        if show_events {
                            for event in effects.events() {
                                print!("{}", textwrap::indent(event, "    "));
                            }
                        }
                        if show_network_effects {
                            for change in effects.resources() {
                                print!("{}", textwrap::indent(&format!("{}", change), "    "));
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        Err(error) => {
            let error = match error.downcast::<ExecCompilerError>() {
                Ok(compiler_error) => {
                    let files_mapping = get_files_for_error_reporting(script, deps);
                    let transformed_errors = compiler_error.transform_with_source_map();
                    report_errors(files_mapping, transformed_errors);
                }
                Err(error) => error,
            };
            Err(error)
        }
    }
}
