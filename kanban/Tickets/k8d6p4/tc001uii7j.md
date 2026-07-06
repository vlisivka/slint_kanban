---
author: user
created_at: 2026-07-05 22:31:16
updated_at: 2026-07-05 22:31:16
---
# План реалізації: Cross-axis alignment — покращення layout

## Проблема
Текст з перекладом виходить за межі кнопок та вікон через відсутність `overflow: elide` на багатьох Text елементах.

## Рішення
Додати `overflow: elide` до всіх Text елементів, що відображають динамічний текст (queue name, ticket title, author, timestamp тощо).

## Зміни

### `ui/components/kanban_column.slint`
- Queue name text (line ~30) — `overflow: elide`
- Points total text (line ~36) — `overflow: elide` (опціонально)

### `ui/components/ticket_card.slint`
- Author label (line ~90) — `overflow: elide`
- Assigned_to label (line ~97) — `overflow: elide`
- Points badge text (line ~109) — `overflow: elide`
- Created/Updated timestamps (lines ~123, ~129) — вже мають overflow

### `ui/dialogs/ticket_view.slint`
- Title label (line ~82) — `overflow: elide`
- Author label (line ~98) — `overflow: elide`
- Assigned_to label (line ~105) — `overflow: elide`

### `ui/dialogs/admin_settings.slint`
- Всі Text елементи з динамічним контентом — `overflow: elide`

## Перевірка
- `cargo test` — всі тести проходять
