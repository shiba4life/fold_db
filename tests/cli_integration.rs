use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use std::fs;
use fold_db::NodeConfig;

#[test]
fn cli_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Setup temporary config
    let dir = tempdir()?;
    let db_dir = dir.path().join("db");
    fs::create_dir_all(&db_dir)?;
    let config = NodeConfig {
        storage_path: db_dir,
        default_trust_distance: 1,
    };
    let config_path = dir.path().join("node_config.json");
    fs::write(&config_path, serde_json::to_string(&config)?)?;
    let cfg = config_path.to_str().unwrap();

    // 1. list-schemas (should be empty)
    Command::cargo_bin("datafold_cli")?
        .args(["-c", cfg, "list-schemas"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded schemas"))
        .stdout(predicate::str::contains("UserProfile").not());

    // 2. load-schema
    Command::cargo_bin("datafold_cli")?
        .args(["-c", cfg, "load-schema", "src/datafold_node/examples/schema1.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Schema loaded successfully"));

    // 3. list-schemas again
    Command::cargo_bin("datafold_cli")?
        .args(["-c", cfg, "list-schemas"])
        .assert()
        .success()
        .stdout(predicate::str::contains("UserProfile"));

    // 4. mutate
    Command::cargo_bin("datafold_cli")?
        .args([
            "-c", cfg,
            "mutate",
            "--schema", "UserProfile",
            "--mutation-type", "create",
            "--data", "{\"username\": \"johndoe\", \"email\": \"john@example.com\"}",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mutation executed successfully"));

    // 5. query
    Command::cargo_bin("datafold_cli")?
        .args([
            "-c", cfg,
            "query",
            "--schema", "UserProfile",
            "--fields", "username,email",
            "--output", "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("johndoe"));

    // 6. execute query from file
    Command::cargo_bin("datafold_cli")?
        .args(["-c", cfg, "execute", "src/datafold_node/examples/query1.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("johndoe"));

    // 7. execute mutation from file
    Command::cargo_bin("datafold_cli")?
        .args(["-c", cfg, "execute", "src/datafold_node/examples/mutation1.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation executed successfully"));

    Ok(())
}
