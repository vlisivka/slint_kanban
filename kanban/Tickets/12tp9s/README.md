---
title: "Story: Відмовитися від поля updated_at на користь mtime файла"
created_at: 2026-07-04 13:22:07
assigned_to: "user"
author: "user"
points: 3
attachment_count: 0
---
# Story: Відмовитися від поля `updated_at` на користь mtime файла

## Контекст

Зараз кожен тікет зберігає час останньої зміни у двох місцях:
- `updated_at` у YAML frontmatter `README.md` (ручно записується програмою)
- `mtime` файлової системи (автоматично оновлюється ОС при зміні файлу)

Це створює проблему для користувачів, які редагують тікети напряму (через текстовий редактор, git, CLU тощо):
- Якщо змінено файл поза програмою — `updated_at` залишається старим і не синхронізується
- Користувачеві доводиться вручну оновлювати `updated_at` у YAML frontmatter
- При commit у git з'являється відволікаючий рядок у diff: `updated_at: "..."`

Файлова система вже має точний час останньої зміни — `mtime`. Його достатньо.

## Гіпотеза вигоди

**Як** користувач або агент, який працює з тікетами напряму (редактор, git, CLU),
**Хочу** щоб час останньої зміни тікета визначався автоматично з файлової системи,
**Щоб** не потрібно було вручну оновлювати `updated_at` і diff у git був чистим.

## Очікувана поведінка

### 1. Збереження (`Ticket::save`)

При збереженні тікета **не записувати** рядок `updated_at` у YAML frontmatter.
Рядок `created_at` залишається — це історичний факт створення тікета.

Замість:
```yaml
---
title: "Task"
created_at: 2026-07-04 13:22:07
updated_at: 2026-07-04 14:30:00
assigned_to: "user"
...
```

Має бути:
```yaml
---
title: "Task"
created_at: 2026-07-04 13:22:07
assigned_to: "user"
...
```

### 2. Завантаження (`Ticket::load`, `Ticket::load_full`)

При завантаженні тікета визначати `updated_at` з `mtime` файлу `README.md`:
- Якщо `mtime` існує — використати його як `updated_at`
- Формат: `"YYYY-MM-DD HH:MM:SS"` (той самий, що й зараз)
- Зберегти backward compatibility — якщо в frontmatter є `updated_at` — ігнорувати його (але не видаляти, щоб не зламати старі тікети під час читання)

### 3. Видалення поля з структур

- Видалити поле `updated_at` зі структури `TicketMetadata`
- Зберегти поле `updated_at` у структурі `Ticket` (для внутрішнього використання UI, статистики тощо), але воно тепер обчислюється з `mtime`, а не читається з файлу

### 4. Документація

Оновити документацію, яка описує формат тікетів:
- `SPECIFICATION.md` — розділ про метадані тікета (прибрати `updated_at` з YAML frontmatter)
- `CODE_STRUCTURE.md` — опис структури даних

## Ризики

- **Backward compatibility**: існуючі тікети мають `updated_at` у frontmatter. Програма має коректно їх читати, але не записувати при збереженні
- **Зміни формату часу**: `mtime` повертає `SystemTime`, який потрібно конвертувати у формат `"YYYY-MM-DD HH:MM:SS"`. Потрібно перевірити, що локальний час співпадає з очікуваннями
- **Тести**: всі тести, які перевіряють наявність/відсутність `updated_at` у YAML frontmatter, потрібно оновити

## Acceptance Criteria

- [x] `Ticket::save()` НЕ записує `updated_at` у YAML frontmatter `README.md`
- [x] `Ticket::load()` і `Ticket::load_full()` визначають `updated_at` з `mtime` файлу `README.md`
- [x] Існуючі тікети з `updated_at` у frontmatter коректно читаються (backward compatibility)
- [x] `TicketMetadata` більше НЕ містить поля `updated_at`
- [x] `Ticket.updated_at` обчислюється з `mtime`, а не з YAML
- [x] При редагуванні тікета через програму `mtime` файлу оновлюється автоматично (звичайна поведінка ОС)
- [x] `cargo test` проходить (всі тести)
- [x] `scripts/pre-commit.sh` проходить (fmt + clippy + tests)
- [x] Документація оновлена (`SPECIFICATION.md`)

## Resolution

### Підсумок

Замінено ручне збереження `updated_at` на автоматичне обчислення з mtime файлу. Видалено поле `updated_at` зі структури `TicketMetadata`.

### Зміни в коді
| Файл | Зміна |
|---|---|
| `src/model/ticket.rs` | Видалено `updated_at` з `TicketMetadata`; додано `system_time_to_string()` для конвертації mtime; `save()` більше не записує `updated_at`; `load()` і `load_full()` обчислюють `updated_at` з `mtime` |
| `src/model/board.rs` | Видалено backfill логіку `metadata.updated_at` з `parse_readme_content()` |
| `src/controller.rs` | Оновлено створення `TicketStr` для board info (прибрано посилання на `metadata.updated_at`) |
| `src/model/tests/stats_tests.rs` | Оновлено виклики `Ticket::from_metadata()` з новим параметром `updated_at` |

### Додані тести
- `test_ticket_save_no_updated_at()` — перевіряє, що `save()` не записує `updated_at` у YAML
- `test_ticket_load_updated_at_from_mtime()` — перевіряє, що `load()` обчислює `updated_at` з mtime файлу
- `test_ticket_load_backward_compat_with_updated_at()` — перевіряє backward compatibility старого формату з `updated_at` у YAML
- Оновлено `test_ticket_metadata_deserialization()` та `test_ticket_metadata_missing_updated_at()` — видалено посилання на поле `metadata.updated_at`

### Оновлена документація
- `SPECIFICATION.md:132` — оновлено опис Ticket metainfo, прибрано `updated_at` з переліку полів YAML frontmatter

## Sources

- Поточна реалізація: `src/model/ticket.rs:10-26` (`TicketMetadata`), `src/model/ticket.rs:28-40` (`Ticket`), `src/model/ticket.rs:139-184` (`load`), `src/model/ticket.rs:205-251` (`load_full`), `src/model/ticket.rs:253-266` (`save`)
- Файлова система: `std::fs::metadata(path)?.modified()` → `SystemTime`