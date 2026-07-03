---
title: "Додавання опису тікету з файлу або stdin в CLI"
created_at: 2026-07-03 09:24:20
updated_at: 2026-07-03 10:20:09
assigned_to: "user"
author: "user"
points: 3
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

- [ ] `slint_kanban add -t "..." -q "1.Incoming" -d "inline text"` працює як раніше
- [ ] `slint_kanban add -t "..." -q "1.Incoming" --description-file body.md` читає з файлу
- [ ] `cat body.md | slint_kanban add -t "..." -q "1.Incoming" -D -` читає з stdin
- [ ] Якщо обидва варіанти вказані (і --description і --description-file), то спочатку береться опис з командного рядка, потім додається `\n`, потім додається опис з файлу.
- [ ] Зрозуміла користувачу помилка при неіснуючому файлі чи закритому stdin.
- [ ] Створено інтеграційні тести для всіх happy path та fail path сценаріїв.
- [ ] Існуючі тести проходять
- [ ] `--help` показує нові прапорці
- [ ] `scripts/pre-commit.sh` проходить.

## Sources

- Поточна реалізація: `src/cli.rs:28-48`, `src/main.rs:326-345`
- Аналогічні інструменти: `git commit -m`, `gh issue create --body-file`, `curl -d @-`