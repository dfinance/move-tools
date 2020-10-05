use std::path::PathBuf;

pub fn get_script_path() -> String {
    get_modules_path().join("script.move").to_str().unwrap().to_owned()
}

// just need some valid fname
pub fn existing_module_file_abspath() -> String {
    std::env::current_dir()
        .unwrap()
        .join("resources")
        .join("modules")
        .join("record.move")
        .into_os_string()
        .into_string()
        .unwrap()
}

pub fn get_test_resources_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .parent() // project root
        .unwrap()
        .join("resources")
        .join("tests")
}

pub fn get_stdlib_path() -> PathBuf {
    get_test_resources_dir().join("stdlib")
}

pub fn get_modules_path() -> PathBuf {
    get_test_resources_dir().join("modules")
}

pub fn setup_test_logging() {
    std::env::set_var("RUST_LOG", "info");
    // silently returns Err if called more than once
    env_logger::builder()
        .is_test(true)
        .try_init()
        .unwrap_or_default();
}

pub fn stdlib_mod(name: &str) -> PathBuf {
    get_stdlib_path().join(name)
}

pub fn modules_mod(name: &str) -> PathBuf {
    get_modules_path().join(name)
}

pub fn stdlib_transaction_mod() -> String {
    stdlib_mod("Transaction.move").to_str().unwrap().to_owned()
}

pub fn record_mod() -> String {
    get_modules_path().join("record.move").to_str().unwrap().to_owned()
}
