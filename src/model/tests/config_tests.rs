//! src/model/tests/config_tests.rs
//!
//! Purpose: Unit tests for Config and search history logic.

use crate::model::config::Config;

#[test]
fn test_search_history() {
    let mut config = Config::default();

    // 1. Add some items
    config.add_search_to_history("rust".to_string());
    config.add_search_to_history("slint".to_string());
    assert_eq!(config.user.search_history, vec!["slint", "rust"]);

    // 2. Add duplicate - should move to top
    config.add_search_to_history("rust".to_string());
    assert_eq!(config.user.search_history, vec!["rust", "slint"]);

    // 3. Limit to 10 items
    for i in 0..15 {
        config.add_search_to_history(format!("search {}", i));
    }
    assert_eq!(config.user.search_history.len(), 10);
    assert_eq!(config.user.search_history[0], "search 14");

    // 4. Ignore empty
    config.add_search_to_history("".to_string());
    assert_eq!(config.user.search_history.len(), 10);

    // 5. Remove item
    config.remove_search_from_history("search 14");
    assert_eq!(config.user.search_history.len(), 9);
    assert!(!config
        .user
        .search_history
        .contains(&"search 14".to_string()));
}

#[test]
fn test_split_config_persistence() {
    use tempfile::tempdir;
    let board_dir = tempdir().unwrap();
    let board_root = board_dir.path();

    // Mock HOME for user config path during test
    let user_dir = tempdir().unwrap();
    std::env::set_var("HOME", user_dir.path());
    // Note: dirs::config_dir() might use different env vars on different OS
    #[cfg(target_os = "linux")]
    std::env::set_var("XDG_CONFIG_HOME", user_dir.path());

    let mut config = Config::default();
    config.kanban.users.push("Alice".to_string());
    config.user.active_user = "Alice".to_string();
    config.set_limit("ToDo".to_string(), 42);
    config.add_search_to_history("rust".to_string());

    // Write it
    config.write(board_root).unwrap();

    // Verify files exist
    assert!(board_root.join("config.toml").exists());
    let user_config_path = Config::user_config_path().expect("Should have user config path");
    assert!(user_config_path.exists());

    // Load it back
    let loaded = Config::load(board_root).unwrap();
    assert_eq!(loaded.kanban.users.len(), 3); // default 2 + Alice
    assert!(loaded.kanban.users.contains(&"Alice".to_string()));
    assert_eq!(loaded.user.active_user, "Alice");
    assert_eq!(loaded.get_limit("ToDo"), Some(42));
    assert_eq!(loaded.user.search_history, vec!["rust"]);
}

#[test]
fn test_machine_id_generation_on_load() {
    use tempfile::tempdir;
    let board_dir = tempdir().unwrap();
    let board_root = board_dir.path();

    let user_dir = tempdir().unwrap();
    std::env::set_var("HOME", user_dir.path());
    #[cfg(target_os = "linux")]
    std::env::set_var("XDG_CONFIG_HOME", user_dir.path());

    let user_config_path = Config::user_config_path().expect("Should have user config path");
    if user_config_path.exists() {
        std::fs::remove_file(&user_config_path).unwrap();
    }

    // Load it - should generate machine_id and save it
    let loaded1 = Config::load(board_root).unwrap();
    assert!(loaded1.machine_id().is_some());
    let machine_id1 = loaded1.machine_id().unwrap().to_string();
    assert!(!machine_id1.is_empty());

    // Verify it saved user config
    assert!(user_config_path.exists());

    // Load it again - should use the saved machine_id
    let loaded2 = Config::load(board_root).unwrap();
    assert_eq!(loaded2.machine_id(), Some(machine_id1.as_str()));
}
