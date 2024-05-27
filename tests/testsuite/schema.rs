#![cfg(feature = "cli")]

#[test]
fn dump_schema() {
    let bin_path = snapbox::cmd::cargo_bin!("git-fixture");
    snapbox::cmd::Command::new(bin_path)
        .arg("--schema")
        .arg("-")
        .assert()
        .success()
        .stdout_eq_(snapbox::file!["../../schema.json"].raw());
}
