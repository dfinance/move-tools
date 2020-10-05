use crate::inner::config::Config;
use crate::global_state::{GlobalStateSnapshot, initialize_new_global_state};
use crate::inner::change::AnalysisChange;
use dialects::file::{MoveFile, read_move_files};

pub fn global_state_snapshot(
    file: MoveFile,
    config: Config,
    additional_files: Vec<MoveFile>,
) -> GlobalStateSnapshot {
    let mut global_state = initialize_new_global_state(config);
    let mut change = AnalysisChange::new();

    for folder in &global_state.config().modules_folders {
        for file in read_move_files(folder) {
            change.add_file(file.path(), file.into_content());
        }
    }

    for file in additional_files {
        change.add_file(file.path(), file.into_content());
    }
    change.update_file(file.path(), file.into_content());

    global_state.apply_change(change);
    global_state.snapshot()
}
