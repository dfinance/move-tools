use std::path::PathBuf;

fn get_tests_dir() -> PathBuf {
    std::env::current_dir().unwrap().join("tests")
}

pub fn get_stdlib_path() -> PathBuf {
    get_tests_dir().join("stdlib")
}

pub fn get_modules_path() -> PathBuf {
    get_tests_dir().join("modules")
}
