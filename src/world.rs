use std::path::PathBuf;

use crossbeam_channel::{unbounded, Receiver};
use ra_vfs::{Filter, RelativePath, RootEntry, Vfs, VfsTask, Watch};

use crate::config::Config;
use crate::ide::analysis::AnalysisHost;
use crate::ide::db::AnalysisChange;
use crate::utils::io::get_module_files;

struct MoveFilesFilter {
    module_folders: Vec<PathBuf>,
}

impl MoveFilesFilter {
    pub fn new(module_folders: Vec<PathBuf>) -> MoveFilesFilter {
        MoveFilesFilter { module_folders }
    }
}

impl Filter for MoveFilesFilter {
    fn include_dir(&self, dir_path: &RelativePath) -> bool {
        let path = dir_path.to_path(std::env::current_dir().unwrap());
        self.module_folders.contains(&path)
    }

    fn include_file(&self, file_path: &RelativePath) -> bool {
        file_path.extension() == Some("move")
    }
}

#[derive(Debug)]
pub struct WorldState {
    pub ws_root: PathBuf,
    pub config: Config,
    pub analysis_host: AnalysisHost,
    pub vfs: Vfs,
    pub fs_events_receiver: Receiver<VfsTask>,
}

impl WorldState {
    pub fn new(ws_root: PathBuf, config: Config) -> WorldState {
        let mut analysis_host = AnalysisHost::default();

        let mut change = AnalysisChange::new();
        for module_folder in &config.module_folders {
            for (fname, text) in get_module_files(module_folder) {
                change.update_file(fname, text);
            }
        }
        change.change_sender_address(config.sender_address);
        analysis_host.apply_change(change);

        let (fs_events_sender, fs_events_receiver) = unbounded::<VfsTask>();
        let modules_root = RootEntry::new(
            ws_root.clone(),
            Box::new(MoveFilesFilter::new(config.module_folders.clone())),
        );
        let vfs = Vfs::new(
            vec![modules_root],
            Box::new(move |task| fs_events_sender.send(task).unwrap()),
            Watch(true),
        )
        .0;

        WorldState {
            ws_root,
            config,
            analysis_host,
            vfs,
            fs_events_receiver,
        }
    }

    pub fn from_old_world_state(world_state: &WorldState, config: Config) -> WorldState {
        WorldState::new(world_state.ws_root.clone(), config)
    }
}
