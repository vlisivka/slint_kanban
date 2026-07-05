---
title: "P0: Замінити ручний Drag & Drop на нативний (DragArea/DropArea)"
created_at: 2026-07-04 14:54:03
assigned_to: "admin"
author: "user"
points: 3
attachment_count: 0
---
## Контекст

Зараз Drag & Drop реалізований вручну через TouchArea з 3-рівневим ланцюжком callback-ів у `ui/components/ticket_card.slint` та `ui/app.slint#363-385`. Це ~400 рядків коду з `is_dragging`, `mouse_x`, `mouse_y`, ghost-Rectangle тощо.

Slint 1.17 додає нативні `DragArea` та `DropArea`, які прибирають більшу частину цього коду.

## Очікувана поведінка

1. Замінити TouchArea-based drag на `DragArea` / `DropArea`
2. Реалізувати Rust-side глобальний API з `make-transfer` / `read-transfer` callback-ами для передачі ID тікета
3. Прибирати `is_dragging`, `mouse_x`, `mouse_y`, ghost-Rectangle
4. Зменшити код з ~400 до ~200 рядків

## Ризики

- Потрібен глобальний Rust API для передачі даних між drag source і drop target
- Можливі зміни у поведінці touch-подій на мобільних пристроях

## Acceptance Criteria

- [x] Drag & Drop карток між колонками працює через `DragArea` / `DropArea`
- [x] Відсутній ручний TouchArea-based drag код
- [x] Rust-side API передає ID тікета коректно
- [x] Всі існуючі тести проходять

## Resolution

### Підсумок

Замінено ручний TouchArea-based Drag & Drop (~400 рядків) на нативні `DragArea` / `DropArea` з Slint 1.17. Реалізовано Rust-side глобальний API для передачі даних між drag source і drop target.

### Зміни в коді
| Файл | Зміна |
|---|---|
| `ui/common.slint` | Додано `export global Api` з `make-transfer`, `can-drop`, `dropped` callback-ами |
| `ui/components/ticket_card.slint` | Замінено TouchArea на DragArea; прибрав callback-и `start-dragging`, `move-dragging`, `drop-ticket` |
| `ui/components/kanban_column.slint` | Додано DropArea з підсвіткою при hover; прибрав callback-и drag; додано `Api` import |
| `ui/app.slint` | Прибрано `is_dragging`, `mouse_x`, `mouse_y`, `dragging_ticket_id`; прибрав ghost-Rectangle і drag callbacks |
| `src/controller.rs` | Додано `DragTransferPayload` struct, `handle_make_transfer`, `handle_can_drop`, `handle_dropped` |
| `src/main.rs` | Додано реєстрацію callback-ів для `Api` global |

### Побічний фікс
- Виправлено стилізацію DropArea: спочатку використовувалося `background` на самому DropArea (не підтримується), замінено на обгортку з `Rectangle`.

### Додані тести
- `test_gui_move_ticket_updates_board()` — вже існував, працює через `invoke_test_trigger_move_ticket`; не потребує змін.
- Всі існуючі тести пройшли (`cargo test` — 72+ тестів, 0 fail).

### Оновлена документація
- Немає (документація проекту не стосується UI-реалізації DnD).