use std::path::{PathBuf, Path};
use crate::manifest::{DoveToml, MANIFEST, read_manifest};
use anyhow::{Result, anyhow};
use std::env;
use lang::compiler::dialects::Dialect;
use lang::compiler::dialects::dfinance::DFinanceDialect;
use lang::compiler::address::ProvidedAccountAddress;

/// Project context.
pub struct Context {
    pub project_dir: PathBuf,
    pub manifest: DoveToml,
    pub dialect: Box<dyn Dialect>,
}

impl Context {
    pub fn path_for<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.project_dir.join(path)
    }

    pub fn project_name(&self) -> String {
        self.manifest.package.name.clone().unwrap_or_else(|| {
            self.project_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("project")
                .to_owned()
        })
    }

    pub fn account_address(&self) -> Result<ProvidedAccountAddress> {
        self.dialect
            .normalize_account_address(&format!("0x{}", &self.manifest.package.account_address))
    }
}

/// Create a new context for the current directory.
pub fn create_context() -> Result<Context> {
    let project_dir = env::current_dir()?;
    Ok(Context {
        project_dir,
        manifest: DoveToml::default(),
        dialect: Box::new(DFinanceDialect::default()),
    })
}

/// Returns project context.
pub fn get_context() -> Result<Context> {
    let project_dir = env::current_dir()?;
    let manifest = load_manifest(&project_dir)?;

    Ok(Context {
        project_dir,
        manifest,
        dialect: Box::new(DFinanceDialect::default()),
    })
}

fn load_manifest(project_dir: &Path) -> Result<DoveToml> {
    let manifest = project_dir.join(MANIFEST);
    if !manifest.exists() {
        Err(anyhow!(
            "could not find `{}` in `{:?}`.",
            MANIFEST,
            project_dir
        ))
    } else {
        read_manifest(&manifest)
    }
}
