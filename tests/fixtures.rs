#[track_caller]
fn assert_success(path: impl AsRef<std::path::Path>) {
    git_fixture::Dag::load(path.as_ref()).unwrap();
}

#[test]
fn branches() {
    assert_success("tests/fixtures/branches.yml");
}

#[test]
fn conflict() {
    assert_success("tests/fixtures/conflict.yml");
}

#[test]
fn fixup() {
    assert_success("tests/fixtures/fixup.yml");
}

#[test]
fn git_rebase_existing() {
    assert_success("tests/fixtures/git_rebase_existing.yml");
}

#[test]
fn git_rebase_new() {
    assert_success("tests/fixtures/git_rebase_new.yml");
}

#[test]
fn pr_semi_linear_merge() {
    assert_success("tests/fixtures/pr-semi-linear-merge.yml");
}

#[test]
fn pr_squash() {
    assert_success("tests/fixtures/pr-squash.yml");
}
