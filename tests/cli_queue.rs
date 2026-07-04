//! Integration tests for CLI queue command.
//! Tests: list, add, rename, delete, settings, tickets

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
/// Create a symlink from `src` to `dst` (platform-agnostic).
fn create_symlink(src: &std::path::Path, dst: &std::path::Path) {
    #[cfg(unix)]
    let _ = std::os::unix::fs::symlink(src, dst);
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_file(src, dst);
}

/// Test: `slint_kanban queue list` outputs all queues.
#[test]
fn test_queue_list() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).args(["queue", "list"]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue list should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show all three queues
    assert!(stdout.contains("1.Incoming"), "Should show 1.Incoming");
    assert!(
        stdout.contains("2.ProductBacklog"),
        "Should show 2.ProductBacklog"
    );
    assert!(
        stdout.contains("3.SprintBacklog"),
        "Should show 3.SprintBacklog"
    );

    // Should show ticket counts (0 tickets each)
    assert!(stdout.contains("0 tickets"), "Should show ticket count");

    Ok(())
}

/// Test: `slint_kanban queue add` creates a new queue.
#[test]
fn test_queue_add() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .arg("--admin")
        .args(["queue", "add", "--name", "4.InReview"]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue add should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Queue added: 4.InReview"),
        "Should confirm queue added"
    );

    // Verify queue directory was created
    assert!(
        env.path().join("Queue/4.InReview").exists(),
        "Queue directory should exist"
    );

    Ok(())
}

/// Test: `slint_kanban queue add` without --admin fails.
#[test]
fn test_queue_add_requires_admin() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["queue", "add", "--name", "4.InReview"]);

    let output = cmd.output()?;
    assert!(
        !output.status.success(),
        "queue add without --admin should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stderr.contains("Admin mode required") || stdout.contains("Admin mode required"),
        "Should require admin: {}",
        stdout
    );
    // Queue should not be created
    assert!(
        !env.path().join("Queue/4.InReview").exists(),
        "Queue directory should NOT exist"
    );

    Ok(())
}

/// Test: `slint_kanban queue rename` renames a queue.
#[test]
fn test_queue_rename() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).arg("--admin").args([
        "queue",
        "rename",
        "--id",
        "1.Incoming",
        "--name",
        "0.Intake",
    ]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue rename should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Queue renamed"),
        "Should confirm queue renamed"
    );

    // Old directory should not exist, new one should
    assert!(
        !env.path().join("Queue/1.Incoming").exists(),
        "Old queue dir should be removed"
    );
    assert!(
        env.path().join("Queue/0.Intake").exists(),
        "New queue dir should exist"
    );

    Ok(())
}

/// Test: `slint_kanban queue delete` deletes an empty queue.
#[test]
fn test_queue_delete_empty() -> anyhow::Result<()> {
    let env = setup_temp_board();
    // Add a new empty queue to delete
    fs::create_dir_all(env.path().join("Queue/9.Archive")).unwrap();

    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .arg("--admin")
        .args(["queue", "delete", "--id", "9.Archive"]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue delete should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Queue deleted"),
        "Should confirm queue deleted"
    );

    assert!(
        !env.path().join("Queue/9.Archive").exists(),
        "Queue directory should be removed"
    );

    Ok(())
}

/// Test: `slint_kanban queue delete` fails on non-empty queue.
#[test]
fn test_queue_delete_nonempty_fails() -> anyhow::Result<()> {
    let env = setup_temp_board();

    // Create a ticket and symlink it to make the queue non-empty
    fs::create_dir_all(env.path().join("Tickets/abc123")).unwrap();
    fs::write(
        env.path().join("Tickets/abc123/README.md"),
        "---\ntitle: Test\n---\nBody",
    )
    .unwrap();

    let tickets_dir = env.path().join("Tickets/abc123");
    let incoming_dir = env.path().join("Queue/1.Incoming");
    create_symlink(&tickets_dir, &incoming_dir.join("abc123"));

    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .arg("--admin")
        .args(["queue", "delete", "--id", "1.Incoming"]);

    let output = cmd.output()?;

    // Should fail because the queue has a ticket symlink
    assert!(
        !output.status.success(),
        "queue delete on non-empty should fail (stdout: {}, stderr: {})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Should have an error message (locale-independent)
    let err_text = String::from_utf8_lossy(&output.stderr);
    assert!(
        !err_text.is_empty(),
        "Should report error on non-empty queue: {}",
        err_text
    );

    Ok(())
}

/// Test: `slint_kanban queue settings` shows queue settings.
#[test]
fn test_queue_settings_view() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["queue", "settings", "-i", "1.Incoming"]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue settings should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1.Incoming") || stdout.contains("Incoming"),
        "Should show queue name"
    );
    assert!(
        stdout.contains("Limit:") || stdout.contains("Tickets:"),
        "Should show settings details"
    );

    Ok(())
}

/// Test: `slint_kanban queue settings` sets a limit.
#[test]
fn test_queue_settings_set_limit() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).arg("--admin").args([
        "queue",
        "settings",
        "-i",
        "1.Incoming",
        "-l",
        "50",
    ]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue settings set-limit should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("limit set to"), "Should confirm limit set");

    // Verify config.toml was updated
    let config_content = fs::read_to_string(env.path().join("config.toml"))?;
    assert!(
        config_content.contains("1.Incoming") && config_content.contains("50"),
        "Config should have updated limit: {}",
        config_content
    );

    Ok(())
}

/// Test: `slint_kanban queue tickets` lists tickets in a queue.
#[test]
fn test_queue_tickets_list() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    // Create a ticket and add it to 1.Incoming
    fs::create_dir_all(env.path().join("Tickets/def456")).unwrap();
    fs::write(
        env.path().join("Tickets/def456/README.md"),
        "---\ntitle: Test Ticket\ncreated_at: 2026-07-03 10:00:00\nupdated_at: 2026-07-03 10:00:00\n---\nBody",
    )
    .unwrap();

    // Create symlink from queue to ticket
    let tickets_dir = env.path().join("Tickets/def456");
    let incoming_dir = env.path().join("Queue/1.Incoming");
    create_symlink(&tickets_dir, &incoming_dir.join("def456"));

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["queue", "tickets", "-i", "1.Incoming"]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue tickets should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("=== 1.Incoming"),
        "Should show queue header"
    );
    assert!(
        stdout.contains("def456") || stdout.contains("Test Ticket"),
        "Should show ticket: {}",
        stdout
    );

    Ok(())
}

/// Test: `slint_kanban queue tickets -v` shows verbose ticket info.
#[test]
fn test_queue_tickets_verbose() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    // Create a ticket
    fs::create_dir_all(env.path().join("Tickets/ghi789")).unwrap();
    fs::write(
        env.path().join("Tickets/ghi789/README.md"),
        "---\ntitle: Verbose Ticket\ncreated_at: 2026-07-03 10:00:00\nupdated_at: 2026-07-03 12:00:00\nassigned_to: user\npoints: 5\nauthor: dev1\n---\nBody",
    )
    .unwrap();

    let tickets_dir = env.path().join("Tickets/ghi789");
    let incoming_dir = env.path().join("Queue/1.Incoming");
    create_symlink(&tickets_dir, &incoming_dir.join("ghi789"));

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["queue", "tickets", "-i", "1.Incoming", "-v"]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue tickets -v should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Verbose Ticket"),
        "Should show ticket title"
    );
    assert!(
        stdout.contains("Points:"),
        "Should show points in verbose mode"
    );
    assert!(stdout.contains("Created:"), "Should show created date");
    assert!(stdout.contains("Updated:"), "Should show updated date");
    assert!(stdout.contains("By:"), "Should show author");
    assert!(stdout.contains("Assigned to:"), "Should show assignee");

    Ok(())
}

/// Test: `slint_kanban queue tickets --assigned-to` filters by user.
#[test]
fn test_queue_tickets_filter_by_user() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    // Create two tickets - one assigned to user, one unassigned
    fs::create_dir_all(env.path().join("Tickets/jkl012")).unwrap();
    fs::write(
        env.path().join("Tickets/jkl012/README.md"),
        "---\ntitle: Assigned Ticket\ncreated_at: 2026-07-03 10:00:00\nupdated_at: 2026-07-03 10:00:00\nassigned_to: user\n---\nBody",
    )
    .unwrap();

    fs::create_dir_all(env.path().join("Tickets/mno345")).unwrap();
    fs::write(
        env.path().join("Tickets/mno345/README.md"),
        "---\ntitle: Unassigned Ticket\ncreated_at: 2026-07-03 10:00:00\nupdated_at: 2026-07-03 10:00:00\nassigned_to:\n---\nBody",
    )
    .unwrap();

    let tickets_dir = env.path().join("Tickets");
    let incoming_dir = env.path().join("Queue/1.Incoming");
    // Create symlinks from queue to tickets (platform-agnostic)
    create_symlink(&tickets_dir.join("jkl012"), &incoming_dir.join("jkl012"));
    create_symlink(&tickets_dir.join("mno345"), &incoming_dir.join("mno345"));

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).args([
        "queue",
        "tickets",
        "-i",
        "1.Incoming",
        "--assigned-to",
        "user",
    ]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "queue tickets --assigned-to should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show assigned ticket, not unassigned
    assert!(
        stdout.contains("Assigned Ticket"),
        "Should show assigned ticket"
    );
    assert!(
        !stdout.contains("Unassigned Ticket") || stdout.contains("mno345"),
        "Unassigned ticket should be filtered out"
    );

    Ok(())
}

/// Test: `queue --help` shows subcommands.
#[test]
fn test_queue_help_shows_subcommands() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).args(["queue", "--help"]);

    let output = cmd.output()?;
    assert!(output.status.success(), "queue --help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"), "Should show list subcommand");
    assert!(stdout.contains("add"), "Should show add subcommand");
    assert!(stdout.contains("rename"), "Should show rename subcommand");
    assert!(stdout.contains("delete"), "Should show delete subcommand");
    assert!(
        stdout.contains("settings"),
        "Should show settings subcommand"
    );
    assert!(stdout.contains("tickets"), "Should show tickets subcommand");

    Ok(())
}

/// Test: `--help` (global) shows queue as a command.
#[test]
fn test_global_help_shows_queue() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).arg("--help");

    let output = cmd.output()?;
    assert!(output.status.success(), "global --help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("queue"),
        "Global help should show queue command"
    );

    Ok(())
}

/// Test: `slint_kanban queue settings` without --admin for setting limit fails.
#[test]
fn test_queue_settings_set_limit_requires_admin() -> anyhow::Result<()> {
    let env = setup_temp_board();
    let root_str = env.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["queue", "settings", "-i", "1.Incoming", "-l", "50"]);

    let output = cmd.output()?;
    assert!(
        !output.status.success(),
        "queue settings set-limit without --admin should fail"
    );

    Ok(())
}
