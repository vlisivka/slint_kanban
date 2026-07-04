# Project Name: Slint Kanban

## Agent Workflow Tips

0. **Don't commit code to git** unless human aproved it and asked to commit it.
1. **Test behavior, not plumbing.** Change defaults shouldn't break tests; assert logical behavior.
2. **Check for contradictions, partial coverage, unique insights, and blind spots** to stay ahead of the loop.
3. **Do not make mistakes.**
4. **Keep documentation in good shape.** If code changed, then it documentation must be updated too. Search for the documentation and update it.
5. **Be accurate**, don't perform destructive actions without explicit permissions. It better to be safe than sorry.
6. **User uses Ukrainian language**, so talk to him in Ukrainian and program must use Ukrainian language when called by user, but code and test cases must use built-in English localization.
7. **Use kanban.sh** to work with tickets. See `./kanban.sh --help` for list of commands.
8. **Use simulate_work** binary to simulate some activity on kanban board, then use slint_kanban binary to work with board. Example: `cargo run --bin simulate_work /tmp/kanban_sim; cargo run --bin slint_kanban -- --root /tmp/kanban_sim stats`
