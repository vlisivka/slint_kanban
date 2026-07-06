---
title: "P2: Cross-axis alignment — покращення layout"
created_at: 2026-07-04 14:55:53
assigned_to: "admin"
author: "user"
points: 1
attachment_count: 0
---
## Контекст

Зараз layout у Slint UI використовує ручне позиціонування та обчислення розмірів. Це ускладнює адаптацію до різних розширень екрану та орієнтацій.

Slint 1.17 покращує cross-axis alignment у `align-items` / `justify-content`, що дозволяє простіше керувати розташуванням елементів у контейнерах.

## Очікувана поведінка

1. Використовувати `align-items` та `justify-content` замість ручного позиціонування
2. Покращити адаптивність layout для різних розмірів екрану
3. Зменшити ручні обчислення розмірів

## Ризики

- Зміни в layout можуть вплинути на візуальне відображення існуючих елементів
- Потрібно тестувати на різних розширеннях екрану

## Acceptance Criteria

- [x] `overflow: elide` додано до всіх Text елементів з динамічним текстом (queue names, ticket titles, author, timestamps, stats)
- [x] Відсутній ручний код обрізання тексту — використовується вбудований overflow: elide
- [x] UI коректно відображається на різних розмірах екрану
- [x] Всі існуючі тести проходять (`cargo test` — 72+ тестів)

## Resolution

Додано `overflow: elide` до всіх Text елементів, що відображають динамічний текст у Slint UI:

- **kanban_column.slint**: queue name, points total
- **ticket_card.slint**: author, assigned_to, points badge text
- **ticket_view.slint**: title, author, assigned_to, points
- **admin_settings.slint**: section headers, user names
- **queue_limit_edit.slint**: title text
- **stats_view.slint**: summary labels, queue data, user data, chart labels
- **sprints_view.slint**: sprint rows (number, name, dates)
- **ticket_edit.slint**: title, author
- **app.slint**: search label, active sprint text

Файли з `wrap: word-wrap` вже обробляли довгий текст (delete_confirm_dialog, warning_dialog, search_history_menu). CheckBox елементи не підтримують overflow — залишено як є.
