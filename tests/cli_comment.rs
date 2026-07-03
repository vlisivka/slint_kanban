//! Integration tests for CLI comment command.
//! Uses assert_cmd which requires CARGO_BIN_EXE_slint_kanban (integration tests only).

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary kanban board structure with a ticket.
fn setup_temp_board_with_ticket() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create minimal board structure with default queues
    fs::create_dir_all(root.join("Queue/1.Incoming")).unwrap();
    fs::create_dir_all(root.join("Queue/2.ProductBacklog")).unwrap();
    fs::create_dir_all(root.join("Queue/3.SprintBacklog")).unwrap();
    fs::create_dir_all(root.join("Tickets")).unwrap();

    // Create a test ticket
    fs::write(
        root.join("config.toml"),
        "[users]\nactive_user = \"user\"\n\n[queue_limits]\n[workflows]\nsprints = []\n",
    )
    .unwrap();

    // Create a ticket that we can comment on
    let ticket_id = "abcd12";
    let ticket_dir = root.join("Tickets").join(ticket_id);
    fs::create_dir_all(&ticket_dir).unwrap();
    fs::write(
        ticket_dir.join("README.md"),
        "---\ntitle: \"Test Ticket\"\ncreated_at: \"2026-07-03 18:00:00\"\nupdated_at: \"2026-07-03 18:00:00\"\nassigned_to: \"\"\nauthor: \"user\"\npoints: 0\nattachment_count: 0\n---\nTest description",
    )
    .unwrap();

    dir
}

/// Test 1: Backward compatible inline text (should work with current code)
#[test]
fn test_cli_comment_inline_text() -> anyhow::Result<()> {
    let env = setup_temp_board_with_ticket();
    let root_str = env
        .path()
        .to_str()
        .expect("tempdir path should be valid utf8");

    // Use assert_cmd to spawn the binary with automatic path resolution
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["comment", "-i", "abcd12", "-c", "Inline comment text"]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify comment was created
    let ticket_dir = env.path().join("Tickets/abcd12");
    let comment_files: Vec<_> = fs::read_dir(&ticket_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            // Comment files match pattern tc<NNN><UID>.md
            name_str.starts_with("tc") && name_str.ends_with(".md")
        })
        .collect();

    assert_eq!(comment_files.len(), 1, "Should have exactly one comment");

    let comment_path = ticket_dir.join(comment_files[0].file_name());
    let content = fs::read_to_string(&comment_path)?;
    assert!(
        content.contains("Inline comment text"),
        "Comment should contain inline text, got: {}",
        content
    );

    Ok(())
}

/// Test 2: Read comment from file (RED - will fail without --content-file)
#[test]
fn test_cli_comment_from_file() -> anyhow::Result<()> {
    let env = setup_temp_board_with_ticket();
    let root_str = env.path().to_str().unwrap();

    // Write comment content to a file
    fs::write(
        env.path().join("comment-body.md"),
        "File comment content for testing",
    )?;

    // Use assert_cmd to spawn the binary
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).args([
        "comment",
        "-i",
        "abcd12",
        "--content-file",
        env.path().join("comment-body.md").to_str().unwrap(),
    ]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify comment was created with file content
    let ticket_dir = env.path().join("Tickets/abcd12");
    let comment_files: Vec<_> = fs::read_dir(&ticket_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("tc") && name_str.ends_with(".md")
        })
        .collect();

    assert_eq!(comment_files.len(), 1, "Should have exactly one comment");

    let comment_path = ticket_dir.join(comment_files[0].file_name());
    let content = fs::read_to_string(&comment_path)?;
    assert!(
        content.contains("File comment content for testing"),
        "Comment should contain file content, got: {}",
        content
    );

    Ok(())
}

/// Test 3: Read comment from stdin using -f -
#[test]
fn test_cli_comment_stdin() -> anyhow::Result<()> {
    let env = setup_temp_board_with_ticket();
    let root_str = env.path().to_str().unwrap();

    // Use assert_cmd to spawn the binary with stdin
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root")
        .arg(root_str)
        .args(["comment", "-i", "abcd12", "-f", "-"])
        .write_stdin("Stdin comment content");

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify comment was created with stdin content
    let ticket_dir = env.path().join("Tickets/abcd12");
    let comment_files: Vec<_> = fs::read_dir(&ticket_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("tc") && name_str.ends_with(".md")
        })
        .collect();

    assert_eq!(comment_files.len(), 1, "Should have exactly one comment");

    let comment_path = ticket_dir.join(comment_files[0].file_name());
    let content = fs::read_to_string(&comment_path)?;
    assert!(
        content.contains("Stdin comment content"),
        "Comment should contain stdin content, got: {}",
        content
    );

    Ok(())
}

/// Test 4: Concatenate inline + file content
#[test]
fn test_cli_comment_concat() -> anyhow::Result<()> {
    let env = setup_temp_board_with_ticket();
    let root_str = env.path().to_str().unwrap();

    // Write comment content to a file
    fs::write(env.path().join("comment-body.md"), "File body content")?;

    // Use assert_cmd to spawn the binary with both -c and -f
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).args([
        "comment",
        "-i",
        "abcd12",
        "-c",
        "Inline prefix",
        "--content-file",
        env.path().join("comment-body.md").to_str().unwrap(),
    ]);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify comment was created with concatenated content
    let ticket_dir = env.path().join("Tickets/abcd12");
    let comment_files: Vec<_> = fs::read_dir(&ticket_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("tc") && name_str.ends_with(".md")
        })
        .collect();

    assert_eq!(comment_files.len(), 1, "Should have exactly one comment");

    let comment_path = ticket_dir.join(comment_files[0].file_name());
    let content = fs::read_to_string(&comment_path)?;
    assert!(
        content.contains("Inline prefix\nFile body content"),
        "Comment should contain concatenated content, got: {}",
        content
    );

    Ok(())
}

/// Test 5: Error on non-existent file
#[test]
fn test_cli_comment_file_not_found() -> anyhow::Result<()> {
    let env = setup_temp_board_with_ticket();
    let root_str = env.path().to_str().unwrap();

    // Use assert_cmd to spawn the binary with non-existent file
    let mut cmd = Command::cargo_bin("slint_kanban")?;
    cmd.arg("--root").arg(root_str).args([
        "comment",
        "-i",
        "abcd12",
        "--content-file",
        "/nonexistent/path/to/file.md",
    ]);

    let output = cmd.output()?;
    assert!(
        !output.status.success(),
        "CLI should fail for non-existent file"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to read comment file") || stderr.contains("No such file"),
        "Should report file not found error, got: {}",
        stderr
    );

    Ok(())
}
