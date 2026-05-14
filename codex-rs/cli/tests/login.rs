use std::path::Path;

use anyhow::Result;
use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;
use tempfile::TempDir;

fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("kodex")?);
    cmd.env("CODEX_HOME", codex_home);
    Ok(cmd)
}

fn write_file_auth_config(codex_home: &Path) -> Result<()> {
    std::fs::write(
        codex_home.join("config.toml"),
        "cli_auth_credentials_store = \"file\"\n",
    )?;
    Ok(())
}

fn read_auth_json(codex_home: &Path) -> Result<Value> {
    let auth_json = std::fs::read_to_string(codex_home.join("auth.json"))?;
    Ok(serde_json::from_str(&auth_json)?)
}

fn api_key_auth_json(api_key: &str) -> String {
    serde_json::json!({
        "OPENAI_API_KEY": api_key,
        "tokens": null,
        "last_refresh": null,
    })
    .to_string()
}

fn trust_project_config(project_dir: &Path) -> String {
    let project_key = serde_json::json!(project_dir.display().to_string());
    format!(
        r#"cli_auth_credentials_store = "file"

[projects.{project_key}]
trust_level = "trusted"
"#
    )
}

#[test]
fn login_with_api_key_reads_stdin_and_writes_auth_json() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_file_auth_config(codex_home.path())?;

    let mut cmd = codex_command(codex_home.path())?;
    cmd.args([
        "-c",
        "forced_login_method=\"api\"",
        "login",
        "--with-api-key",
    ])
    .write_stdin("sk-test\n")
    .assert()
    .success()
    .stderr(contains("Successfully logged in"));

    let auth = read_auth_json(codex_home.path())?;
    assert_eq!(auth["OPENAI_API_KEY"], "sk-test");
    assert!(auth.get("tokens").is_none());
    assert!(auth.get("agent_identity").is_none());

    Ok(())
}

#[test]
fn login_status_prefers_project_auth_json_over_global_auth_json() -> Result<()> {
    let codex_home = TempDir::new()?;
    let project_dir = TempDir::new()?;
    std::fs::write(
        codex_home.path().join("config.toml"),
        trust_project_config(project_dir.path()),
    )?;
    std::fs::write(
        codex_home.path().join("auth.json"),
        api_key_auth_json("sk-global-1234567890ABCDE"),
    )?;
    let project_codex_dir = project_dir.path().join(".codex");
    std::fs::create_dir_all(&project_codex_dir)?;
    std::fs::write(
        project_codex_dir.join("auth.json"),
        api_key_auth_json("sk-project-1234567890ABCDE"),
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    cmd.current_dir(project_dir.path())
        .arg("login")
        .arg("status")
        .assert()
        .success()
        .stderr(contains("sk-proje***ABCDE"));

    Ok(())
}

#[test]
fn logout_removes_project_auth_json_without_removing_global_auth_json() -> Result<()> {
    let codex_home = TempDir::new()?;
    let project_dir = TempDir::new()?;
    std::fs::write(
        codex_home.path().join("config.toml"),
        trust_project_config(project_dir.path()),
    )?;
    std::fs::write(
        codex_home.path().join("auth.json"),
        api_key_auth_json("sk-global-1234567890ABCDE"),
    )?;
    let project_codex_dir = project_dir.path().join(".codex");
    std::fs::create_dir_all(&project_codex_dir)?;
    let project_auth_file = project_codex_dir.join("auth.json");
    std::fs::write(
        &project_auth_file,
        api_key_auth_json("sk-project-1234567890ABCDE"),
    )?;

    let mut cmd = codex_command(codex_home.path())?;
    cmd.current_dir(project_dir.path())
        .arg("logout")
        .assert()
        .success()
        .stderr(contains("Successfully logged out"));

    assert!(!project_auth_file.exists());
    assert!(codex_home.path().join("auth.json").exists());
    Ok(())
}

#[test]
fn login_with_access_token_rejects_invalid_jwt() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_file_auth_config(codex_home.path())?;

    let mut cmd = codex_command(codex_home.path())?;
    cmd.args(["login", "--with-access-token"])
        .write_stdin("not-a-jwt\n")
        .assert()
        .failure()
        .stderr(contains("Error logging in with access token"));

    Ok(())
}
