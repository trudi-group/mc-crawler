use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn minimal_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mc-crawler")?;
    cmd.assert().success();
    Ok(())
}

#[test]
fn invalid_bootstrap_file_passed() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mc-crawler")?;
    cmd.arg("./invalid-file");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error opening bootstrap file"));
    Ok(())
}
