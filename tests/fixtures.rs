#![cfg(feature = "yaml")]

#[track_caller]
fn assert_success(name: &str) {
    let _ = env_logger::try_init();

    let path = std::path::PathBuf::from(format!("tests/fixtures/{name}.yml"));

    let dag = git_fixture::TodoList::load(&path).unwrap();

    let tmpdir = std::path::Path::new(std::env!("CARGO_TARGET_TMPDIR"));
    let sandbox = tmpdir.join("test").join("case").join(name);
    if sandbox.exists() {
        std::fs::remove_dir_all(&sandbox).unwrap();
    }
    std::fs::create_dir_all(&sandbox).unwrap();
    dag.run(&sandbox).unwrap();
}

#[test]
fn branches() {
    assert_success("branches");
}

#[test]
fn conflict() {
    assert_success("conflict");
}

#[test]
fn fixup() {
    assert_success("fixup");
}

#[test]
fn git_rebase_existing() {
    assert_success("git_rebase_existing");
}

#[test]
fn git_rebase_new() {
    assert_success("git_rebase_new");
}

#[test]
fn pr_semi_linear_merge() {
    assert_success("pr-semi-linear-merge");
}

#[test]
fn pr_squash() {
    assert_success("pr-squash");
}
