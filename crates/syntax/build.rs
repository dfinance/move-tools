use std::path::PathBuf;

fn compile_parser(resources_dir: PathBuf) {
    let move_parser_file =
        std::fs::canonicalize(resources_dir.join("parser.c")).expect(&format!(
            "No such file or directory {:?}",
            resources_dir
                .join("parser.c")
                .into_os_string()
                .into_string()
                .unwrap()
        ));

    println!("cargo:rerun-if-changed={}", resources_dir.to_str().unwrap()); // <1>

    cc::Build::new()
        .file(move_parser_file)
        .include(resources_dir)
        .compile("tree-sitter-move");
}

fn main() {
    let resources_dir = std::env::current_dir().unwrap().join("resources");
    compile_parser(resources_dir);
}
