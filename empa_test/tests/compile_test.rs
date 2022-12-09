use compiletest_rs::{run_tests, Config};
use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = Config::default();

    config.mode = mode.parse().expect("Invalid mode");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.target_rustcflags = Some(String::from(
        "\
         --edition=2021 \
         -Z unstable-options \
         -Z macro-backtrace \
         --extern empa \
         ",
    ));
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path
    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

    run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("run-pass");
}
