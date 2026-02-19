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
    assert_eq!(config.search_history, vec!["slint", "rust"]);

    // 2. Add duplicate - should move to top
    config.add_search_to_history("rust".to_string());
    assert_eq!(config.search_history, vec!["rust", "slint"]);

    // 3. Limit to 10 items
    for i in 0..15 {
        config.add_search_to_history(format!("search {}", i));
    }
    assert_eq!(config.search_history.len(), 10);
    assert_eq!(config.search_history[0], "search 14");

    // 4. Ignore empty
    config.add_search_to_history("".to_string());
    assert_eq!(config.search_history.len(), 10);

    // 5. Remove item
    config.remove_search_from_history("search 14");
    assert_eq!(config.search_history.len(), 9);
    assert!(!config.search_history.contains(&"search 14".to_string()));
}
