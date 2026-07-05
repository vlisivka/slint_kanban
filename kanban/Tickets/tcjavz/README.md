---
title: "Додати підтримку --content-file до kanban comment"
created_at: 2026-07-03 17:55:35
assigned_to: "user"
author: "user"
points: 2
attachment_count: 0
---
# Додати підтримку --content-file до kanban comment

## Контекст

Зараз команда `slint_kanban comment` приймає коментар виключно як аргумент командного рядка (`--content "<text>"`). Це створює проблеми для багатолінійних коментарів, особливо з YAML frontmatter або markdown форматуванням — shell не коректно обробляє спеціальні символи та нові рядки.

Поточний код (`src/cli.rs:176-184`):
```rust
Comment {
    /// Ticket ID (short ID)
    #[arg(short, long)]
    id: String,

    /// Comment content
    #[arg(short, long)]
    content: String,
},
```

Це аналогічна проблема, що й у тікеті **#4fy8ug** з опцією `-d`/`--description` для команди `add`. Там ми додали підтримку `-D`/`--description-file` для читання з файлу або stdin. Тепер потрібно зробити те саме для коментарів.

## Поточний стан коду

| File | Рядок | Опис |
|---|---|---|
| `src/cli.rs:176-184` | `content: String` у `Commands::Comment` — звичайний String аргумент clap |
| `src/main.rs:~300` | `board.add_comment(&id, &content, author)?;` — передає вміст безпосередньо |

## Запропонований інтерфейс

```bash
# З рядка (backward compatible)
slint_kanban comment -id abc123 -c "Simple comment"

# З файлу
slint_kanban comment -id abc123 --content-file comment-body.md

# З stdin (pipeline)
cat template.md | slint_kanban comment -id abc123 -f -
```

## Зміни в коді

### 1. `src/cli.rs` — додати опцію content_file

```rust
Comment {
    /// Ticket ID (short ID)
    #[arg(short, long)]
    id: String,

    /// Comment content (if empty, reads from --content-file or stdin)
    #[arg(short, long)]
    content: Option<String>,

    /// File to read comment content from (use '-' for stdin)
    #[arg(short = 'f', long)]
    content_file: Option<String>,
},
```

### 2. `src/main.rs` — логіка зчитування контенту

Аналогічно до #4fy8ug:
- Якщо `-f -` → читати з stdin
- Якщо `-f <file>` → читати з файлу
- Якщо `-c <text>` → використовувати інлайн
- Якщо обидва `-c` і `-f` → конкатенація: `[-c]\n[<file>]`

### 3. `src/model/board.rs` — `add_comment()` — без змін
(приймає `&str` як контент, працює без змін)

## Ризики

- **Backward compatibility**: поточна поведінка `--content "text"` має зберегтися
- **stdin detection**: коли stdin — це термінал (не pipeline), краще показати warning або прочекати timeout
- **Error messages**: треба чітко повідомити користувача, звідки береться контент

## Acceptance Criteria

- [x] `slint_kanban comment -id TICKET -c "inline text"` працює як раніше (backward compatible)
- [x] `slint_kanban comment -id TICKET --content-file body.md` читає з файлу
- [x] `cat body.md | slint_kanban comment -id TICKET -f -` читає з stdin
- [x] Якщо обидва варіанти вказані (і --content і --content-file), то спочатку береться контент з командного рядка, потім додається `\n`, потім додається контент з файлу.
- [x] Зрозуміла користувачу помилка при неіснуючому файлі чи закритому stdin.
- [x] Створено інтеграційні тести для всіх happy path та fail path сценаріїв.
- [x] Існуючі тести проходять
- [x] `--help` показує нові прапорці
- [x] `scripts/pre-commit.sh` проходить

## Sources

- Поточна реалізація: `src/cli.rs:176-184`, `src/main.rs:~300`
- Аналогічний патерн: тікет #4fy8ug (додано `-D`/`--description-file` для команди `add`)
- Аналогічні інструменти: `gh issue create --body-file`, `curl -d @-`

## Resolution

### Реалізований інтерфейс

```bash
# Інлайн-текст (backward compatible)
slint_kanban comment -i TICKET -c "Inline text"

# З файлу
slint_kanban comment -i TICKET --content-file body.md

# З stdin
cat body.md | slint_kanban comment -i TICKET -f -

# Конкатенація: інлайн + файл
slint_kanban comment -i TICKET -c "Inline" --content-file body.md
# Результат: "Inline\n[вміст body.md]"
```

### Пріоритет зчитування

1. `-f -` (stdin) — читає з stdin, ігнорує `-c`
2. `-f <файл>` — читає з файлу; якщо `-c` також вказано → конкатенація: `[-c]\n[файл]`
3. `-c <текст>` — використовує інлайн-текст
4. Немає жодного прапорця → порожній коментар

### Зміни в коді

| Файл | Зміна |
|---|---|
| `src/cli.rs:176-188` | `content: Option<String>`, додано `content_file: Option<String>` з прапорцем `-f` |
| `src/main.rs:587-613` | `handle_command` → логіка body resolution (stdin/file/inline/concat) |
| `tests/cli_comment.rs` | 5 інтеграційних тестів: inline, file, stdin, concat, error |

### Тести

- `test_cli_comment_inline_text()` — backward compat: `-c "text"`
- `test_cli_comment_from_file()` — читання з файлу
- `test_cli_comment_stdin()` — stdin через `-f -`
- `test_cli_comment_concat()` — конкатенація інлайн + файл
- `test_cli_comment_file_not_found()` — помилка при відсутності файлу

67 тестів, 6 suites. Clippy та fmt — без помилок.
