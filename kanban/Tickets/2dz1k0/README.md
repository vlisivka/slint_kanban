---
title: "Task: Об'єднати дубльовані допоміжні функції у спільний файл"
created_at: 2026-07-04 09:20:56
assigned_to: "user"
author: "user"
points: 2
attachment_count: 0
---
# Task: Об'єднати дубльовані допоміжні функції у спільний файл

## Контекст

Під час рефакторингу було виявлено дублювання допоміжних функцій між різними модулями. Це ускладнює підтримку: зміни потрібно вносити в кілька місць, зростає ризик розбіжностей.

## Аналіз дублювання

### 1. `extract_references` — ВИКЛИКАЄ ДУБЛЮВАННЯ

Функція для пошуку посилань на тікети у форматі `#xxxxxx` (6 символів, `[a-z0-9]`) повністю ідентична в:
- `src/model/ticket.rs:60-82` (`Ticket::extract_references`) — працює з `self.description`
- `src/model/comment.rs:24-46` (`Comment::extract_references`) — працює з `self.content`

Алгоритм однаковий: `char_indices()` → пошук `#` → взяти 6 символів → перевірка алфавіту → сортування + dedup.

**Пропозиція:** Створити спільну функцію `extract_ticket_references(text: &str) -> Vec<String>` у новому файлі `src/model/utils.rs`. Обидва методи стають обгортками.

### 2. YAML frontmatter parsing — ЧАСТКОВЕ ДУБЛЮВАННЯ

`Ticket::load()` (ticket.rs:139-184) та `Ticket::load_full()` (ticket.rs:205-251) мають ідентичний блок парсингу frontmatter (lines 158-167 і 224-233):
```rust
let parts: Vec<&str> = content.splitn(3, "---").collect();
// ... validation ...
let frontmatter = parts[1];
let body = parts[2].trim().to_string();
let mut metadata: TicketMetadata = serde_yaml::from_str(frontmatter)?;
```

**Пропозиція:** Створити `parse_frontmatter(content: &str) -> Result<(String, String), anyhow::Error>` у `utils.rs`. Викликати з обох методів.

### 3. `save()` — НЕ ВИКЛИКАЄ ДУБЛЮВАННЯ

`Ticket::save()` (ticket.rs:253-266) та `Comment::save()` (comment.rs:90-101) мають схожу структуру (YAML + markdown), але формат frontmatter відрізняється (`serde_yaml::to_string` vs ручне форматування). **Не варто об'єднувати.**

## Очікуваний результат

Створити `src/model/utils.rs` з:
```rust
/// Extract ticket references from text.
/// Finds patterns like #abc123 where x is [a-z0-9]{6}.
pub fn extract_ticket_references(text: &str) -> Vec<String> { ... }

/// Parse YAML frontmatter from a document.
/// Returns (frontmatter_yaml, body_markdown).
pub fn parse_frontmatter(content: &str) -> Result<(String, String), anyhow::Error> { ... }
```

Оновити виклики у `Ticket` та `Comment`. Задокументувати кожен helper.

## Ризики

- **Backward compatibility**: функції мають бути бінарно сумісними — той самий вхід, той самий вихід
- **Тести**: потрібно перенести існуючі тести з `Ticket::extract_references` та `Comment::extract_references` у новий модуль або додати тестові обгортки

## Acceptance Criteria

- [ ] Створено `src/model/utils.rs` з функцією `extract_ticket_references(text: &str) -> Vec<String>`
- [ ] `Ticket::extract_references()` викликає `utils::extract_ticket_references(&self.description)`
- [ ] `Comment::extract_references()` викликає `utils::extract_ticket_references(&self.content)`
- [ ] Створено функцію `parse_frontmatter(content: &str) -> Result<(String, String), anyhow::Error>`
- [ ] `Ticket::load()` та `Ticket::load_full()` використовують `parse_frontmatter`
- [ ] Кожна функція в `utils.rs` має doc-comment з описом призначення
- [ ] `cargo test` проходить (всі тести)
- [ ] `scripts/pre-commit.sh` проходить (fmt + clippy + tests)

## Sources

- `src/model/ticket.rs:60-82` — `Ticket::extract_references`
- `src/model/comment.rs:24-46` — `Comment::extract_references`
- `src/model/ticket.rs:158-167` — парсинг frontmatter у `load()`
- `src/model/ticket.rs:224-233` — парсинг frontmatter у `load_full()`
