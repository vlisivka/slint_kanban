//! Integration tests for CLI add command with stdin description.
//! Uses assert_cmd which requires CARGO_BIN_EXE_slint_kanban (integration tests only).

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary kanban board structure and return its root path.
fn setup_temp_board() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create minimal board structure with default queues
    fs::create_dir_all(root.join("Queue/1.Incoming")).unwrap();
    fs::create_dir_all(root.join("Queue/2.ProductBacklog")).unwrap();
    fs::create_dir_all(root.join("Queue/3.SprintBacklog")).unwrap();
    fs::create_dir_all(root.join("Tickets")).unwrap();

    fs::write(
        root.join("config.toml"),
        "[users]\nactive_user = \"user\"\n\n[queue_limits]\n[workflows]\nsprints = []\n",
    )
    .unwrap();

    dir
}

/// Test reading description from stdin using -D -
#[test]
fn test_cli_add_stdin_description() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env
        .path()
        .to_str()
        .expect("tempdir path should be valid utf8");

    // Use assert_cmd to spawn the binary with automatic path resolution
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["add", "-t", "Stdin Ticket", "-q", "1.Incoming", "-D", "-"])
        .write_stdin("Stdin description content");

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The CLI creates symlinks in Queue/<queue>/<ticket_id> -> Tickets/<ticket_id>/README.md
    // We need to find the ticket directory in Tickets/ and read its README.md
    let tickets_dir = env.path().join("Tickets");
    let ticket_dirs: Vec<_> = fs::read_dir(&tickets_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            // Ticket IDs are 6 alphanumeric chars
            name_str.len() == 6 && name_str.chars().all(|c| c.is_alphanumeric())
        })
        .collect();

    assert_eq!(
        ticket_dirs.len(),
        1,
        "Should have exactly one ticket in Tickets/"
    );

    let readme_path = tickets_dir
        .join(ticket_dirs[0].file_name().to_string_lossy().to_string())
        .join("README.md");

    assert!(
        readme_path.exists(),
        "README.md should exist at {:?}",
        readme_path
    );

    let content = fs::read_to_string(&readme_path)?;
    assert!(
        content.contains("Stdin description content"),
        "Ticket should contain stdin content, got: {}",
        content
    );

    Ok(())
}
