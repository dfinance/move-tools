use std::path::PathBuf;

use crate::config::Config;
use crate::ide::analysis::AnalysisHost;

#[derive(Debug)]
pub struct WorldState {
    pub ws_root: PathBuf,
    pub config: Config,
    pub analysis_host: AnalysisHost,
}

impl WorldState {
    pub fn new(ws_root: PathBuf, config: Config) -> WorldState {
        let analysis_host = AnalysisHost::default();

        // TODO: initialize filesystem watcher
        // TODO: load all files from ws_root and modules_roots
        // TODO: apply_change()

        WorldState {
            ws_root,
            config,
            analysis_host,
        }
    }

    pub fn from_old_world_state(world_state: &WorldState, config: Config) -> WorldState {
        WorldState::new(world_state.ws_root.clone(), config)
    }
}
