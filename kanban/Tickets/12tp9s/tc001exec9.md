---
author: user
created_at: 2026-07-05 10:22:52
---
# План виконання #12tp9s — Відмова від `updated_at` на користь mtime

## Зміни в коді

### 1. `src/model/ticket.rs`

**Змінити структуру `TicketMetadata`** (рядки 10-26):
- Видалити поле `pub updated_at: String`
- Залишити `created_at`, `title`, `assigned_to`, `author`, `points`, `attachment_count`

**Змінити `Ticket::save()`** (рядки 253-266):
- Прибрати `updated_at` з формату YAML frontmatter у `write!()`
- Залишити тільки `title`, `created_at`, `assigned_to`, `author`, `points`, `attachment_count`

**Змінити `Ticket::load()`** (рядки 139-184):
- Додати зчитування `mtime` файлу README.md через `std::fs::metadata(path)?.modified()`
- Конвертувати `SystemTime` у формат `"YYYY-MM-DD HH:MM:SS"` (локальний час)
- Використовувати цей час замість `metadata.updated_at` при створенні Ticket
- Якщо `updated_at` вже є в YAML — ігнорувати його (backward compatibility)

**Змінити `Ticket::load_full()`** (рядки 205-251):
- Аналогічно до `load()` — обчислювати `updated_at` з mtime

**Додати helper-функцію** для конвертації `SystemTime` → `"YYYY-MM-DD HH:MM:SS"`:
```rust
fn system_time_to_string(time: SystemTime) -> String {
    // конвертує SystemTime в локальний час у форматі YYYY-MM-DD HH:MM:SS
}
```

### 2. `src/model/tests/ticket_tests.rs`

**Видалити/оновити тест `test_ticket_metadata_missing_updated_at`**:
- Оскільки поле видалено з `TicketMetadata`, перевірка `metadata.updated_at == ""` більше не має сенсу
- Можна видалити або змінити на перевірку, що інші поля працюють

**Додати новий тест `test_ticket_save_no_updated_at`**:
- Зберегти тікет → завантажити → перевірити, що в YAML немає `updated_at`

**Додати тест `test_ticket_load_updated_at_from_mtime`**:
- Створити тимчасовий тікет з конкретним `created_at`
- Встановити `mtime` файлу на відомий час через `std::time::SystemTime`
- Завантажити → перевірити, що `ticket.updated_at` = мtime

**Додати тест `test_ticket_load_backward_compat`**:
- Створити README.md з `updated_at` у frontmatter
- Завантажити — має спрацювати (ігнорує поле)

### 3. Документація

**Оновити `SPECIFICATION.md`** (рядок 132):
- Змінити: "Ticket metainfo: is stored in README.md file in YAML format. Contains `title`, `created_at`, `updated_at`, `assigned_to`, `author`, and `points`."
- На: "Ticket metainfo: is stored in README.md file in YAML format. Contains `title`, `created_at`, `assigned_to`, `author`, and `points`. The `updated_at` field is computed from the file's mtime and NOT stored in frontmatter."

## Acceptance Criteria

- [x] `Ticket::save()` НЕ записує `updated_at` у YAML frontmatter
- [x] `Ticket::load()` і `Ticket::load_full()` визначають `updated_at` з `mtime` файлу README.md
- [x] Існуючі тікети з `updated_at` у frontmatter коректно читаються (backward compatibility)
- [x] `TicketMetadata` більше НЕ містить поля `updated_at`
- [x] `Ticket.updated_at` обчислюється з mtime, а не з YAML
- [x] При редагуванні тікета через програму `mtime` файлу оновлюється автоматично
- [x] `cargo test` проходить (всі тести)
- [x] `scripts/pre-commit.sh` проходить (fmt + clippy + tests)
- [x] Документація оновлена (`SPECIFICATION.md`)
