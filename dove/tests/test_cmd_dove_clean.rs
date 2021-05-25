mod helper;
use crate::helper::{project_start_new_and_build, project_remove, execute_dove_at};

/// $ dove clean
#[test]
fn test_cmd_dove_clean() {
    // Path to dove folder, project and project name
    let project_name = "demoproject_16";
    let project_folder = project_start_new_and_build(project_name);

    // $ dove clean
    execute_dove_at(&project_folder, &["dove", "clean"]).unwrap_or_else(|err| {
        panic!("{}", err);
    });
    let project_target = project_folder.join("target");
    assert!(
        !project_target.exists(),
        "Directory was not deleted: {}",
        project_target.display()
    );

    project_remove(&project_folder);
}