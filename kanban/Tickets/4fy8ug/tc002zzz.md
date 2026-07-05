---
author: user
created_at: 2026-07-03 11:00:00
---
Тікет завершено. Реалізовано:
1. `-d`/`--description` — інлайн текст (backward compatible)
2. `-D`/`--description-file <path>` — читання з файлу
3. `-D -` — читання з stdin
4. Конкатенація `-d "текст" -D file.md` → "текст\n[вміст файлу]"
5. Помилка при неіснуючому файлі: "Failed to read description file '...'"

Змінено файли:
- `src/cli.rs` — `description: Option<String>`, додано `description_file: Option<String>`
- `src/main.rs` — логіка body resolution (stdin/file/inline/concat)
- `src/model/ticket.rs` — виправлено `Ticket::load()` (читав лише перший рядок тіла)
- `src/main_tests.rs` — 4 нових інтеграційних тести

Всі 62 тести проходять, clippy та fmt без помилок.
