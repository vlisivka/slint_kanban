---
title: "Crash on Ctrl-M (toggle 'show only mine' filter)"
created_at: 2026-07-06 09:27:43
assigned_to: "admin"
author: "user"
points: 1
attachment_count: 0
---
## Summary

Програма падає з паником при натисканні `Ctrl-M` для ввімкнення фільтра "тільки мої тікети".

## Type: Bug

## Severity: High (blocks workflow — user cannot use the filter)

## Steps to Reproduce

1. Запустити програму:
   ```
   target/debug/slint_kanban --root=/home/vlisivka/workspace/slint_kanban/kanban
   ```
2. Натиснути `Ctrl-M` для ввімкнення фільтра "тільки мої тікети" (show only mine).
3. Програма падає з паником:
   ```
   thread 'main' panicked at /home/vlisivka/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/i-slint-core-1.17.0/model.rs:398:41:
   removal index (is 2) should be < len (is 2)
   ```

## Expected Result

Фільтр "тільки мої тікети" має застосовуватися, відображаючи лише тікети поточного користувача. Програма не має падати.

## Actual Result

Програма падає з паником у `VecModel::remove()`:
```
removal index (is 2) should be < len (is 2)
```

## Environment

- ОС: Linux (Fedora, x86_64)
- Версія Rust: стандартна (з Cargo registry)
- Slint: 1.17
- Пакет: `i-slint-core-1.17.0`
- Трейс стеку: `controller::AppController::reload` → `VecModel::remove`

## Impact

Блокує використання фільтра "тільки мої тікети" (`Ctrl-M`). Користувач не може відсортувати тікети за автором, що є критичною функцією для роботи з великими дошками.

## Root Cause Analysis

Баг у `src/controller.rs` рядки 177–181 (метод `sync_board_data`):

```rust
// Shrink the model to remove stale rows
for _ in 0..(current_len - new_len) {
    tickets_model.remove(current_len - 1);
}
```

Цикл видаляє елементи з фіксованим індексом `current_len - 1`, але після кожного виклику `remove()` довжина вектора зменшується на 1. Третя ітерація (або пізніша) намагається видалити індекс, який вийшов за межі поточної довжини — звідси паник `removal index (is 2) should be < len (is 2)`.

**Виправлення:** використовувати динамічний індекс — `tickets_model.remove(current_len - 1 - i)` або `new_len + i`, де `i` — лічильник ітерації циклу.

## Acceptance Criteria

- [x] Натискання `Ctrl-M` не викликає панику
- [x] Фільтр "тільки мої тікети" коректно застосовується
- [x] Фільтр працює для всіх користувачів (включно з "<unassigned>")
- [x] Повернення фільтра (`Ctrl-M` знову) також без панику
- [x] Існуючі тести проходять (91 тест, 0 помилок)

## Resolution

### Підсумок

Виправлено баг у `src/controller.rs` — паник при натисканні `Ctrl-M` для фільтра "тільки мої тікети".

### Зміни в коді
| Файл | Зміна |
|---|---|
| `src/controller.rs:179-180` | Виправлено цикл `VecModel::remove()` — тепер використовується динамічний індекс `current_len - 1 - i` замість фіксованого `current_len - 1` |
| `src/gui_tests.rs:347-405` | Додано регресійний тест `test_gui_toggle_show_only_mine_no_crash()` в `test_gui_suite` |

### Додані тести
- `test_gui_toggle_show_only_mine_no_crash()` — відтворює сценарій з 3+ тікетами в одній черзі, перемикає `show_only_mine` кілька разів, перевіряє що не падає і фільтр застосовується коректно

### Побічний фікс
- `test_gui_move_ticket_updates_board()` — повернено виклик у `test_gui_suite` (випадково видалено під час редагування)
