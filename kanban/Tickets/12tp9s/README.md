---
title: "Story: Відмовитися від поля updated_at на користь mtime файла"
created_at: 2026-07-04 13:22:07
updated_at: 2026-07-04 14:28:10
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

- [ ] `Ticket::save()` НЕ записує `updated_at` у YAML frontmatter `README.md`
- [ ] `Ticket::load()` і `Ticket::load_full()` визначають `updated_at` з `mtime` файлу `README.md`
- [ ] Існуючі тікети з `updated_at` у frontmatter коректно читаються (backward compatibility)
- [ ] `TicketMetadata` більше НЕ містить поля `updated_at`
- [ ] `Ticket.updated_at` обчислюється з `mtime`, а не з YAML
- [ ] При редагуванні тікета через програму `mtime` файлу оновлюється автоматично (звичайна поведінка ОС)
- [ ] `cargo test` проходить (всі тести)
- [ ] `scripts/pre-commit.sh` проходить (fmt + clippy + tests)
- [ ] Документація оновлена (`SPECIFICATION.md`чи інші файли)

## Sources

- Поточна реалізація: `src/model/ticket.rs:10-26` (`TicketMetadata`), `src/model/ticket.rs:28-40` (`Ticket`), `src/model/ticket.rs:139-184` (`load`), `src/model/ticket.rs:205-251` (`load_full`), `src/model/ticket.rs:253-266` (`save`)
- Файлова система: `std::fs::metadata(path)?.modified()` → `SystemTime`