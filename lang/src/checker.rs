use crate::compiler::dialects::Dialect;
use crate::compiler::address::ProvidedAccountAddress;
use crate::compiler::file::MvFile;
use crate::compiler::errors::{CompilerError, ExecCompilerError};
use crate::compiler::{CompileFlow, Step, compile, CheckerResult};
use crate::compiler::parser::{ParsingMeta, ParserArtifact};
use move_lang::compiled_unit::CompiledUnit;
use move_lang::errors::Errors;

pub struct MoveChecker<'a> {
    dialect: &'a dyn Dialect,
    sender: Option<&'a ProvidedAccountAddress>,
}

impl<'a> MoveChecker<'a> {
    pub fn new(
        dialect: &'a dyn Dialect,
        sender: Option<&'a ProvidedAccountAddress>,
    ) -> MoveChecker<'a> {
        MoveChecker { dialect, sender }
    }

    pub fn check(
        self,
        targets: Vec<MvFile>,
        deps: Vec<MvFile>,
    ) -> Result<(), Vec<CompilerError>> {
        compile(self.dialect, targets, deps, self.sender, self)
    }
}

impl<'a> CompileFlow<Result<(), Vec<CompilerError>>> for MoveChecker<'a> {
    fn after_parsing(
        &mut self,
        parser_artifact: ParserArtifact,
    ) -> Step<Result<(), Vec<CompilerError>>, ParserArtifact> {
        if parser_artifact.result.is_err() {
            let ParserArtifact { meta, result } = parser_artifact;
            Step::Stop(result.map(|_| ()).map_err(|errors| {
                ExecCompilerError::new(errors, meta.offsets_map).transform_with_source_map()
            }))
        } else {
            Step::Next(parser_artifact)
        }
    }

    fn after_check(
        &mut self,
        meta: ParsingMeta,
        check_result: CheckerResult,
    ) -> Step<Result<(), Vec<CompilerError>>, (ParsingMeta, CheckerResult)> {
        Step::Stop(check_result.map(|_| ()).map_err(|errors| {
            ExecCompilerError::new(errors, meta.offsets_map).transform_with_source_map()
        }))
    }

    fn after_translate(
        &mut self,
        _: ParsingMeta,
        _: Result<Vec<CompiledUnit>, Errors>,
    ) -> Result<(), Vec<CompilerError>> {
        Ok(())
    }
}