use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Error;
use fs_extra::dir::CopyOptions;
use git2::{Oid, Repository};
use git2::build::RepoBuilder;
use tiny_keccak::{Hasher, Sha3};

use crate::context::Context;
use crate::manifest::{CheckoutParams, Git};
use crate::stdoutln;

/// Git prefix.
pub const PREFIX: &str = "git";

/// Returns module path by its identifier.
/// Downloads a modules from git if it is not in the cache.
pub fn resolve(ctx: &Context, git: &Git) -> Result<PathBuf, Error> {
    let checkout_params = CheckoutParams::try_from(git)?;

    let deps = ctx.path_for(&ctx.manifest.layout.deps);
    let local_name = make_local_name(&git);
    let mut repo_path = deps.join(&local_name);

    if !repo_path.exists() {
        if git.path.is_some() {
            repo_path = repo_path.join("._tmp_dove_checkout_dir_");
        }

        if let Err(err) = checkout(checkout_params, &repo_path) {
            fs::remove_dir_all(&repo_path)?;
            return Err(err);
        }

        if let Err(err) = fs::remove_dir_all(&repo_path.join(".git")) {
            warn!("Failed to remove .git in repo {:?}. {}", repo_path, err);
        }

        if let Some(path_in_repo) = &git.path {
            let source_path = repo_path
                .join(&path_in_repo)
                .canonicalize()
                .map_err(|err| anyhow!("Invalid path in git repo.{} [{}]", git.git, err))?;

            if !source_path.starts_with(&repo_path) {
                if let Some(repo_path) = repo_path.parent() {
                    fs::remove_dir_all(&repo_path)?;
                }
                return Err(anyhow!(
                    "Invalid path in git repo.{} [Path is output of git directory]",
                    git.git
                ));
            }
            let target_path = deps.join(&local_name);
            if source_path.is_file() {
                fs::copy(source_path, target_path)?;
            } else {
                for entry in fs::read_dir(source_path)? {
                    let source_path = entry?.path();
                    if source_path.is_file() {
                        if let Some(name) = source_path.file_name() {
                            fs::copy(&source_path, target_path.join(name))?;
                        }
                    } else {
                        fs_extra::dir::move_dir(&source_path, &target_path, &CopyOptions::new())?;
                    }
                }
            }
            fs::remove_dir_all(&repo_path)?;
        }
    }

    Ok(repo_path)
}

fn checkout(params: CheckoutParams<'_>, path: &Path) -> Result<(), Error> {
    let repo = clone(&params, path)?;
    match params {
        CheckoutParams::Branch { repo: _, branch } => {
            if let Some(branch_name) = branch {
                let refs = format!("refs/remotes/origin/{}", branch_name);

                let head = repo.head()?;
                let oid = head
                    .target()
                    .ok_or_else(|| anyhow!("Failed to take repo {} head.", params.repo()))?;
                let commit = repo.find_commit(oid)?;

                repo.branch(branch_name, &commit, false)?;
                let obj = repo.revparse_single(&refs)?;
                repo.checkout_tree(&obj, None)?;
                repo.set_head(&refs)?;
            }
        }
        CheckoutParams::Rev { repo: _, rev } => {
            let oid = Oid::from_str(rev)?;
            let commit = repo.find_commit(oid)?;

            repo.branch(rev, &commit, false)?;

            let refs = format!("refs/heads/{}", rev);

            let obj = repo.revparse_single(&refs)?;
            repo.checkout_tree(&obj, None)?;
            repo.set_head(&refs)?;
        }
        CheckoutParams::Tag {
            repo: _,
            tag: tg_name,
        } => {
            let references = repo.references()?;

            let refs = format!("refs/tags/{}", tg_name);

            for reference in references.flatten() {
                if reference.is_tag() {
                    if let Some(tag_ref) = reference.name() {
                        if tag_ref == refs {
                            let commit = reference.peel_to_commit()?;
                            repo.branch(tg_name, &commit, false)?;
                            let obj = repo.revparse_single(&refs)?;
                            repo.checkout_tree(&obj, None)?;
                            repo.set_head(&refs)?;
                            return Ok(());
                        }
                    }
                }
            }
            return Err(anyhow!("Tag {} not found.", tg_name));
        }
    }
    Ok(())
}

/// Returns unique repository name for git repository.
pub fn make_local_name(git: &Git) -> String {
    let mut digest = Sha3::v256();
    digest.update(git.git.as_bytes());
    if let Some(branch) = &git.branch {
        digest.update(branch.as_bytes());
    }
    if let Some(rev) = &git.rev {
        digest.update(rev.as_bytes());
    }
    if let Some(path) = &git.path {
        digest.update(path.as_bytes());
    }
    if let Some(tag) = &git.tag {
        digest.update(tag.as_bytes());
    }
    let mut output = [0; 32];
    digest.finalize(&mut output);
    format!("{}_{}", PREFIX, hex::encode(&output))
}

fn clone(git: &CheckoutParams, path: &Path) -> Result<Repository, Error> {
    stdoutln!("Download:[{}]", git.repo());
    RepoBuilder::new()
        .clone(&git.repo(), path)
        .map_err(|err| anyhow!("Failed to clone repository :[{}]:{}", git.repo(), err))
}
