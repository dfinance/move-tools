use crate::context::Context;
use anyhow::Error;
use std::fs;
use toml::Value;
use std::fs::OpenOptions;
use std::io::Write;
use serde::{Serialize, Deserialize};
use crate::index::Index;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use libra::prelude::ModuleId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Modules {
    pub modules: Vec<Module>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ModulesRef<'a> {
    pub modules: Vec<&'a Module>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SourceType {
    Local,
    Git,
    Chain,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Module {
    pub name: Rc<ModuleId>,
    pub dep_name: Rc<String>,
    pub path: Rc<String>,
    pub source_type: SourceType,
    pub dependencies: HashSet<Rc<ModuleId>>,
}

impl<'a> Index<'a> {
    pub fn load(ctx: &'a Context) -> Result<Index<'a>, Error> {
        let index_path = ctx.path_for(&ctx.manifest.layout.index);
        if index_path.exists() {
            let index = toml::from_str::<Modules>(&fs::read_to_string(index_path)?)?;

            let dep_names = index.modules.iter().map(|m| m.dep_name.clone()).collect();

            let modules = index.modules.into_iter().map(|m| (m.name.clone(), m)).fold(
                HashMap::new(),
                |mut acc, (name, m)| {
                    let entry = acc.entry(name).or_insert_with(HashMap::default);
                    entry.insert(m.source_type, m);
                    acc
                },
            );

            Ok(Index {
                modules,
                dep_names,
                ctx,
            })
        } else {
            Ok(Index {
                modules: Default::default(),
                dep_names: Default::default(),
                ctx,
            })
        }
    }

    pub fn store(&self) -> Result<(), Error> {
        let modules: Vec<&Module> = self
            .modules
            .iter()
            .map(|(_, module)| module)
            .flat_map(|m| m.values())
            .collect();

        let modules = ModulesRef { modules };

        let value = toml::to_vec(&Value::try_from(modules)?)?;

        let path = self.ctx.path_for(&self.ctx.manifest.layout.index);

        let mut f = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        f.set_len(0)?;
        f.write_all(&value)?;

        Ok(())
    }
}
