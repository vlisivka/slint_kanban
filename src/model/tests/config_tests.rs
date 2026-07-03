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

    let user_dir = tempdir().unwrap();
    Config::set_test_user_config_path(Some(user_dir.path().join("user.toml")));

    let mut config = Config::default();
    config.kanban.users.push("Alice".to_string());
    config.user.active_user = "Alice".to_string();
    config.set_limit("To Do".to_string(), 42);
    config.add_search_to_history("rust".to_string());
    config.user.manage_only_mine = false; // Override default

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
    assert_eq!(loaded.get_limit("To Do"), Some(42));
    assert_eq!(loaded.user.search_history, vec!["rust"]);
    assert!(!loaded.manage_only_mine());
}

#[test]
fn test_machine_id_generation_on_load() {
    use tempfile::tempdir;
    let board_dir = tempdir().unwrap();
    let board_root = board_dir.path();

    let user_dir = tempdir().unwrap();
    Config::set_test_user_config_path(Some(user_dir.path().join("user.toml")));

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

#[test]
fn test_get_current_sprint() {
    use crate::model::config::Sprint;
    let mut config = Config::default();

    let today = chrono::Local::now().naive_local().date();
    let yesterday = (today - chrono::Duration::days(1)).to_string();
    let tomorrow = (today + chrono::Duration::days(1)).to_string();
    let last_week = (today - chrono::Duration::days(7)).to_string();
    let two_weeks_ago = (today - chrono::Duration::days(14)).to_string();

    config.kanban.sprints = vec![
        Sprint {
            number: 1,
            name: "Past Sprint".to_string(),
            start_date: two_weeks_ago,
            end_date: last_week,
        },
        Sprint {
            number: 2,
            name: "Current Sprint".to_string(),
            start_date: yesterday,
            end_date: tomorrow,
        },
    ];

    let current = config
        .get_current_sprint(None)
        .expect("Should find current sprint");
    assert_eq!(current.number, 2);
    assert_eq!(current.name, "Current Sprint");

    // Test with specific date
    let past = config
        .get_current_sprint(Some(&config.kanban.sprints[0].start_date))
        .expect("Should find past sprint");
    assert_eq!(past.number, 1);
}
