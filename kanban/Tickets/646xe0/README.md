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

- [ ] Drag & Drop карток між колонками працює через `DragArea` / `DropArea`
- [ ] Відсутній ручний TouchArea-based drag код
- [ ] Rust-side API передає ID тікета коректно
- [ ] Всі існуючі тести проходять
