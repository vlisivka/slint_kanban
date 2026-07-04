---
title: "Enabler: підтримка .keepme файлів для git"
created_at: 2026-07-03 23:02:30
updated_at: 2026-07-04 13:39:58
assigned_to: "user"
author: "user"
points: 2
attachment_count: 0
---
# Enabler: підтримка `.keepme` файлів для git

## Контекст

Git не зберігає порожні каталоги. Коли є каталоги-контейнери (наприклад `Queue/`, `Tickets/`, `logs/`), які можуть залишатися порожніми, вони зникають з репозиторію після першого commit без файлів у цих теках. Це створює проблеми при клонуванні — каталоги не існують, і програма має створювати їх знову.

Поточний стан:
- Каталоги `Queue/`, `Tickets/` не мають `.keepme` файлів
- При порожньому каталозі git не відстежує його

## Гіпотеза вигоди

**Як** розробник, який працює з git-репозиторієм slint_kanban,
**Хочу** мати стабільну структуру каталогів `Queue/`, `Tickets/`, `logs/` у репозиторії,
**Щоб** при клонуванні проекту структура була збережена без необхідності створювати каталоги вручну.

## Очікувана поведінка

### 1. Ініціалізація (`Board::ensure_initialized`)

При виклику `Board::ensure_initialized()` — якщо каталог існує, але порожній, додати `.keepme` файл:
- Створювати `.keepme` тільки в кореневих теках: `Queue/`, `Tickets/`, `logs/`
- Якщо каталог вже містить файли (навіть якщо це `.keepme`) — нічого не робити
- `.keepme` — порожній файл

### 2. Створення черги (`Board::add_queue`)

При виклику `board.add_queue(name)` — створювати `.keepme` у новому каталозі черги:
- Створювати `.keepme` у `Queue/<queue_id>/.keepme`
- Викликати `std::fs::write(&keepme_path, "")` після створення каталогу

### 3. Ігнорування dotfiles

Програма **не повинна** видаляти або змінювати файли/каталоги, що починаються з `.`:
- `Board::delete_queue()` — перевіряти, чи є в каталозі тільки `.*` (тоді порожній) або є інші файли (тоді не порожній)
- При скануванні каталогу — фільтрувати entries, що починаються з `.`

## Ризики

- **Backward compatibility**: існуючі каталоги без `.keepme` чи з `.keepme` мають працювати як раніше
- **Порожній каталог після видалення тікетів**: програма не моніторить каталоги і не відновлює `.keepme` — це очікувана поведінка (користувач видалив все — каталог порожній)
- **Dotfiles користувача**: якщо користувач створив власні файли `.something`, вони не видалятимуться і не заважатимуть роботі

## Acceptance Criteria

- [x] Створення нового робочого каталогу Kanban створює `.keepme` у `Queue/`, `Tickets/`, `logs/` при першій ініціалізації
- [x] `Board::add_queue()` створює `.keepme` у новій черзі (`Queue/<id>/.keepme`)
- [x] `delete_queue()` вважає каталог порожнім навіть якщо він містить тільки `.keepme` (і інші dotfiles)
- [x] Сканування каталогів (queue/tickets) ігнорує dotfiles — не розглядає їх як тікети/черги, та **не видаляє їх**.
- [x] `cargo test` проходить (всі тести)
- [x] `scripts/pre-commit.sh` проходить (fmt + clippy + tests)
- [x] Документація перевірена і оновлена.

## Resolution

### Зміни в коді
| Файл | Зміна |
|---|---|
| `src/model/board.rs` | Додано helper `has_non_dot_entries()` для перевірки наявності не-dotfile записів |
| `src/model/board.rs` | `ensure_initialized()` тепер створює `.keepme` у Queue/, Tickets/, logs/ через `ensure_keepme()` |
| `src/model/board.rs` | `add_queue()` створює `.keepme` у новій черзі |
| `src/model/board.rs` | `delete_queue()` видаляє каталоги з dotfiles через `remove_dir_all` |
| `src/model/board.rs` | Всі `read_dir` цикли фільтрують dotfiles (.keepme, .DS_Store тощо) |
| `src/model/tests/board_tests.rs` | Додано 6 нових тестів для .keepme та dotfile фільтрації |

### Додані тести
- `test_ensure_initialized_creates_keepme()` — перевірка .keepme у кореневих каталогах
- `test_add_queue_creates_keepme()` — перевірка .keepme у новій черзі
- `test_delete_queue_with_only_dotfiles()` — видалення черги з dotfiles всередині
- `test_delete_queue_with_non_dotfiles_fails()` — помилка при видаленні непорожньої черги
- `test_scan_queues_ignores_dotfile_dirs()` — сканування ігнорує каталоги-файли приховані
- `test_scan_tickets_ignores_dotfile_entries()` — сканування ігнорує dotfiles у чергах

### Примітка
Існуючі черги (4.InProgress, 1.Draft, тощо) не мають `.keepme` всередині — вони створені до реалізації цієї функції. Кожна нова черга автоматично отримає `.keepme`.

## Sources

- Поточна реалізація: `src/model/board.rs:119-166` (`ensure_initialized`), `src/model/board.rs:169-175` (`ensure_keepme`), `src/model/board.rs:200-221` (`add_queue`), `src/model/board.rs:261-293` (`delete_queue`)
- Git behavior: https://git-scm.com/book/en/v2/Git-Basics-Recording-Changes-to-the-Repository#_ignoring