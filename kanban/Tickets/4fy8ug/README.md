---
title: "Додавання опису тікету з файлу або stdin в CLI"
created_at: 2026-07-03 09:24:20
assigned_to: "user"
author: "user"
points: 1
attachment_count: 0
---
# Додавання опису тікету з файлу або stdin в CLI

## Контекст

Зараз команда `slint_kanban add` приймає опис тікету виключно як аргумент командного рядка (`--description <text>`). Це створює проблеми:

1. **Довгі описи ламають CLI** — багатолінійний текст з YAML frontmatter не можна передати через `--description` через обмеження shell-екранування (помилка `unexpected argument '---`).
2. **Неможливо зчитати з файлу** — якщо опис зберігається у файлі (наприклад, згенерований іншим інструментом), його доводиться копіювати в буфер обміну або екранувати вручну.
3. **Неможливо зчитати з stdin** — не підтримується pipeline-стиль наприклад `cat template.md | slint_kanban add --queue ...`.

Поточний код (`src/cli.rs:34-35`):
```rust
#[arg(short, long, default_value = "")]
description: String,
```

Це звичайний String аргумент clap — він приймає лише один токен з командного рядка.

## Поточний стан коду

| File | Рядок | Опис |
|---|---|---|
| `src/cli.rs` | 34-35 | `description: String` у `Commands::Add` — звичайний String аргумент clap |
| `src/main.rs` | 329, 345 | Опис передається як `&description` у `board.create_ticket()` |

## Запропонований інтерфейс

 `-D` / `--description-file <path>` (використовуй `-` для stdin)

```bash
# З файлу
slint_kanban add -t "Fix login bug" -q "1.Incoming" -D ticket-body.md

# З stdin
cat ticket-body.md | slint_kanban add -t "Fix login bug" -q "1.Incoming" -D -
```


## Зміни в коді

### 1. `src/cli.rs` — змінити тип description

```rust
// Замість:
#[arg(short, long, default_value = "")]
description: String,

// На:
/// Description text (if empty, reads from --description-file or stdin)
#[arg(short, long)]
description: Option<String>,

/// File to read description from (use '-' for stdin)
#[arg(long)]
description_file: Option<String>,
```

### 2. `src/main.rs` — логіка зчитування опису

```rust
Commands::Add {
    title,
    description,
    description_file,
    queue,
    assign_to,
    points,
} => {
    let body = match (description_file, description) {
        (Some(ref path), _) if path != "-" => {
            std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?
        }
        (_, Some(desc)) => desc,
        _ => {
            // Read from stdin
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input)?;
            input
        }
    };
    
    board.create_ticket(&title, &body, &queue, &assign_to, author, points)?;
}
```

### 3. `src/model/board.rs` — `create_ticket()` — без змін
(приймає `&str` як опис, працює без змін)

## Ризики

- **Backward compatibility**: поточна поведінка `--description "text"` має зберегтися
- **stdin detection**: коли stdin — це термінал (не pipeline), краще показати warning або прочекати timeout
- **Error messages**: треба чітко повідомити користувача, звідки береться опис

## Acceptance Criteria

- [x] `slint_kanban add -t "..." -q "1.Incoming" -d "inline text"` працює як раніше
- [x] `slint_kanban add -t "..." -q "1.Incoming" --description-file body.md` читає з файлу
- [x] `cat body.md | slint_kanban add -t "..." -q "1.Incoming" -D -` читає з stdin
- [x] Якщо обидва варіанти вказані (і --description і --description-file), то спочатку береться опис з командного рядка, потім додається `\n`, потім додається опис з файлу.
- [x] Зрозуміла користувачу помилка при неіснуючому файлі чи закритому stdin.
- [x] Створено інтеграційні тести для всіх happy path та fail path сценаріїв.
- [x] Існуючі тести проходять (62 test, 4 suites)
- [x] `--help` показує нові прапорці
- [x] `scripts/pre-commit.sh` проходить (fmt + clippy + tests)
- [x] Перевірити, чи потрібне оновлення документації (SPECIFICATION.md, README.md, CODE-STRUCTURE.md). Якщо так — оновити і згадати в Resolution comments.

## Sources

- Поточна реалізація: `src/cli.rs:28-48`, `src/main.rs:326-345`
- Аналогічні інструменти: `git commit -m`, `gh issue create --body-file`, `curl -d @-`

## Resolution comments

### Реалізований інтерфейс

```bash
# Інлайн-опис (backward compatible)
slint_kanban add -t "Title" -q "1.Incoming" -d "description text"

# З файлу
slint_kanban add -t "Title" -q "1.Incoming" -D body.md

# З stdin
cat body.md | slint_kanban add -t "Title" -q "1.Incoming" -D -

# Конкатенація: інлайн + файл
slint_kanban add -t "Title" -q "1.Incoming" -d "inline" -D body.md
# Результат: "inline\n[вміст body.md]"
```

### Пріоритет зчитування

1. `-D -` (stdin) — читає з stdin, ігнорує `-d`
2. `-D <файл>` — читає з файлу; якщо `-d` також вказано → конкатенація: `[-d]\n[файл]`
3. `-d <текст>` — використовує інлайн-текст
4. Немає жодного прапорця → порожній опис

### Зміни в коді

| Файл | Зміна |
|---|---|
| `src/cli.rs:33-39` | `description: Option<String>`, додано `description_file: Option<String>` з прапорцем `-D` |
| `src/main.rs:327-368` | `handle_command` → логіка body resolution (stdin/file/inline/concat) |
| `src/model/ticket.rs:139-184` | Виправлено `Ticket::load()` — читання всього body, а не першого рядка |
| `src/main_tests.rs:441-560` | 4 інтеграційних тести: file, stdin, concat, error |
| `tests/cli_add_stdin.rs` — інтеграційний тест stdin через `assert_cmd::cargo_bin()`

### Побічний фікс

Баг у `Ticket::load()`: метод читав лише перший рядок Markdown-тіла після YAML frontmatter. Багатолінійні описи тікетів обрізалися до одного рядка. Замінено state-machine парсер на `splitn(3, "---")` — узгоджено з `Ticket::load_full()`.

### Інфраструктура тестів

Тест `test_cli_add_stdin_description` переміщено з `src/main_tests.rs` (unit test) у `tests/cli_add_stdin.rs` (integration test), оскільки `assert_cmd::cargo_bin()` вимагає `CARGO_BIN_EXE_slint_kanban`, який доступний тільки для інтеграційних тестів. Використано `assert_cmd::Command::cargo_bin()` замість ручного пошуку бінарника.

### Тести

- `test_cli_add()` — backward compat: `-d "inline text"`
- `test_cli_add_description_file()` — читання з файлу
- `test_cli_add_stdin_description()` — stdin через `-D -` (підпроцес)
- `test_cli_add_description_concat()` — конкатенація інлайн + файл
- `test_cli_add_description_file_not_found()` — помилка при відсутності файлу

62 тести, 5 suites. Clippy та fmt — без помилок.

### Документація

Оновлено `SPECIFICATION.md` — додано розділ **CLI Commands** (після Non-Functional Requirements, перед Future Enhancements). Розділ містить повний довідник всіх CLI команд з описом прапорців, прикладами використання та таблицею пріоритетів для `-d`/`-D`.