use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

const SCHEMA_JSON: &str = r#"{
    "name": "TestSchema",
    "fields": {
        "username": {
            "field_type": "Single",
            "permission_policy": {
                "read_policy": {"Distance": 0},
                "write_policy": {"Distance": 0},
                "explicit_read_policy": null,
                "explicit_write_policy": null
            },
            "payment_config": {
                "base_multiplier": 1.0,
                "trust_distance_scaling": {"None": null},
                "min_payment": null
            },
            "field_mappers": {}
        }
    },
    "payment_config": {
        "base_multiplier": 1.0,
        "min_payment_threshold": 0
    }
}"#;

fn setup_files() -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let dir = TempDir::new().expect("temp dir");
    let db_dir = dir.path().join("db");
    fs::create_dir_all(&db_dir).unwrap();

    let config_path = dir.path().join("config.json");
    let config_content = format!(
        "{{\n  \"storage_path\": \"{}\",\n  \"default_trust_distance\": 1,\n  \"network_listen_address\": \"/ip4/127.0.0.1/tcp/0\"\n}}",
        db_dir.display()
    );
    fs::write(&config_path, config_content).unwrap();

    let schema_path = dir.path().join("schema.json");
    fs::write(&schema_path, SCHEMA_JSON).unwrap();

    let op_path = dir.path().join("operation.json");
    fs::write(
        &op_path,
        "{\"type\":\"query\",\"schema\":\"TestSchema\",\"fields\":[\"username\"],\"filter\":null}",
    )
    .unwrap();

    (dir, config_path, schema_path, op_path)
}

fn cli_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest_dir.join("target/debug/datafold_cli");

    if !exe.exists() {
        // Build the CLI binary if it hasn't been compiled yet
        let status = Command::new("cargo")
            .args([
                "build",
                "--manifest-path",
                "fold_node/Cargo.toml",
                "--bin",
                "datafold_cli",
            ])
            .status()
            .expect("failed to build datafold_cli");
        assert!(status.success());
    }

    exe
}

#[test]
fn load_and_list_schemas() {
    let (_dir, config, schema, _) = setup_files();
    let exe = cli_path();

    let status = Command::new(&exe)
        .args(["-c", config.to_str().unwrap(), "load-schema", schema.to_str().unwrap()])
        .status()
        .expect("load-schema command failed");
    assert!(status.success());

    let output = Command::new(&exe)
        .args(["-c", config.to_str().unwrap(), "list-schemas"])
        .output()
        .expect("list-schemas command failed");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TestSchema"));
}

#[test]
fn mutate_query_execute() {
    let (_dir, config, schema, op) = setup_files();
    let exe = cli_path();

    let status = Command::new(&exe)
        .args(["-c", config.to_str().unwrap(), "load-schema", schema.to_str().unwrap()])
        .status()
        .expect("load-schema");
    assert!(status.success());

    let status = Command::new(&exe)
        .args([
            "-c",
            config.to_str().unwrap(),
            "mutate",
            "--schema",
            "TestSchema",
            "--mutation-type",
            "create",
            "--data",
            "{\"username\":\"alice\"}",
        ])
        .status()
        .expect("mutate");
    assert!(status.success());

    let output = Command::new(&exe)
        .args([
            "-c",
            config.to_str().unwrap(),
            "query",
            "--schema",
            "TestSchema",
            "--fields",
            "username",
            "--output",
            "json",
        ])
        .output()
        .expect("query");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"alice\""));

    let output = Command::new(&exe)
        .args(["-c", config.to_str().unwrap(), "execute", op.to_str().unwrap()])
        .output()
        .expect("execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"alice\""));
}

#[test]
fn help_and_version() {
    let exe = cli_path();

    let output = Command::new(&exe)
        .arg("--help")
        .output()
        .expect("failed to run --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));

    let output = Command::new(&exe)
        .arg("--version")
        .output()
        .expect("failed to run --version");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_config_fails() {
    let exe = cli_path();
    let status = Command::new(&exe)
        .args(["-c", "nonexistent.json", "list-schemas"])
        .status()
        .expect("command failed");
    assert!(!status.success());
}
