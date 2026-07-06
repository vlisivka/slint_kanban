---
author: user
created_at: 2026-07-06 12:54:09
updated_at: 2026-07-06 12:54:09
---
# План виконання

## Підсумок

Створити `src/model/utils.rs` з двома універсальними функціями, які замінять дубльований код у Ticket та Comment.

## Кроки

1. **Створити `src/model/utils.rs`**
   - `extract_ticket_references(text: &str) -> Vec<String>` — перенести алгоритм з `Ticket::extract_references` і `Comment::extract_references`
   - `parse_frontmatter(content: &str) -> Result<(String, String), anyhow::Error>` — перенести спільний блок парсингу YAML frontmatter з `Ticket::load()` і `Ticket::load_full()`
   - Додати doc-comment до кожної функції

2. **Додати модуль у `src/model/mod.rs`**
   - `pub mod utils;`

3. **Оновити `Ticket::extract_references()`** (ticket.rs:69-91)
   - Викликати `utils::extract_ticket_references(&self.description)`

4. **Оновити `Comment::extract_references()`** (comment.rs:24-46)
   - Викликати `utils::extract_ticket_references(&self.content)`

5. **Оновити `Ticket::load()`** (ticket.rs:147-191)
   - Замінити блок парсингу frontmatter на `utils::parse_frontmatter(&content)`

6. **Оновити `Ticket::load_full()`** (ticket.rs:212-257)
   - Замінити блок парсингу frontmatter на `utils::parse_frontmatter(&content)`

## Ацептанс-критерії
- [x] Створено `src/model/utils.rs` з функцією `extract_ticket_references(text: &str) -> Vec<String>`
- [ ] `Ticket::extract_references()` викликає `utils::extract_ticket_references(&self.description)`
- [ ] `Comment::extract_references()` викликає `utils::extract_ticket_references(&self.content)`
- [ ] Створено функцію `parse_frontmatter(content: &str) -> Result<(String, String), anyhow::Error>`
- [ ] `Ticket::load()` та `Ticket::load_full()` використовують `parse_frontmatter`
- [ ] Кожна функція в `utils.rs` має doc-comment з описом призначення
- [ ] `cargo test` проходить (всі тести)
- [ ] `scripts/pre-commit.sh` проходить (fmt + clippy + tests)
