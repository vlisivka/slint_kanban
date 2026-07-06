//! Integration tests for CLI stats command.
//! Verifies statistics output doesn't contain format string garbage.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary kanban board structure.
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
        "[users]\nactive_user = \"user\"\n\n[queue_limits]\n\"1.Incoming\" = 0\n\"2.ProductBacklog\" = 21\n\"3.SprintBacklog\" = 5\n[workflows]\nsprints = []\n",
    )
    .unwrap();

    dir
}

/// Test that stats output for empty board doesn't contain format string garbage.
#[test]
fn test_cli_stats_no_format_garbage() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    // Run stats command
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).arg("stats");

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "Stats command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that format string patterns are NOT present in output
    assert!(
        !stdout.contains("{:<20}"),
        "Output should not contain format string garbage '{{:<20}}':\n{}",
        stdout
    );
    assert!(
        !stdout.contains("{:>5}"),
        "Output should not contain format string garbage '{{:>5}}':\n{}",
        stdout
    );
    assert!(
        !stdout.contains("{:10}"),
        "Output should not contain format string garbage '{{:10}}':\n{}",
        stdout
    );

    // Verify expected headers ARE present
    assert!(
        stdout.contains("Тікетів у чергах") || stdout.contains("Tickets per Queue"),
        "Output should contain queue section header"
    );

    Ok(())
}

/// Regression test for ticket a1wxcn: tr!() strips format specifiers, leaving {:.1} in output.
#[test]
fn test_cli_stats_no_format_specifier_garbage() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    // Run stats command
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).arg("stats");

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "Stats command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that {:.1} format specifier patterns are NOT present in output
    assert!(
        !stdout.contains("{:.1}"),
        "Output should not contain {{:.1}} format string garbage:\n{}",
        stdout
    );

    Ok(())
}
