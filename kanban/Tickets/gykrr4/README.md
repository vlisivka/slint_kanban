---
title: "P1: Two-way model bindings замість editing_* properties"
created_at: 2026-07-04 14:55:19
updated_at: 2026-07-04 14:55:19
assigned_to: "admin"
author: "user"
points: 2
attachment_count: 0
---
## Контекст

Зараз редагування тікетів використовує 6 ручних `editing_*` property та state copy у `ui/app.slint#94-101` та `ui/dialogs/ticket_edit.slint`. Кожен field має свій pair property для двонаправленого зв'язування.

Slint 1.17 додає `in property <TicketStr> ticket` з two-way model bindings, що дозволяє замінити всі `editing_*` property на пряме двонаправлене зв'язування з моделлю.

## Очікувана поведінка

1. Замінити 6 `editing_*` properties на `in property <TicketStr> ticket`
2. Використовувати two-way model bindings для синхронізації полів
3. Прибирати ручну копію state та синхронізацію

## Ризики

- Two-way bindings на `in property` (не `model`) можуть не працювати як очікується
- Потрібна ретельна перевірка сумісності з існуючою логікою валідації

## Acceptance Criteria

- [ ] Редагування тікета використовує `in property <TicketStr> ticket` замість `editing_*` properties
- [ ] Two-way model bindings коректно синхронізують зміни між UI та моделлю
- [ ] Валідація полів працює без збоїв
- [ ] Всі існуючі тести проходять
