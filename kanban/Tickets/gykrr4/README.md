---
title: "P1: Two-way model bindings замість editing_* properties"
created_at: 2026-07-04 14:55:19
assigned_to: "admin"
author: "user"
points: 2
attachment_count: 0
---
## Контекст

Зараз редагування тікетів використовує 6 ручних `editing_*` property та state copy у `ui/app.slint#94-101` та `ui/dialogs/ticket_edit.slint`. Кожен field має свій pair property для двонаправленого зв'язування.

Slint 1.17 додає `in-out property <TicketStr> ticket` з two-way model bindings, що дозволяє замінити всі `editing_*` property на пряме двонаправлене зв'язування з моделлю.

## Очікувана поведінка

1. Замінити 6 `editing_*` properties на `in-out property <TicketStr> ticket`
2. Використовувати two-way model bindings для синхронізації полів
3. Прибирати ручну копію state та синхронізацію

## Ризики

- Two-way bindings на `in property` (не `model`) можуть не працювати як очікується
- Потрібна ретельна перевірка сумісності з існуючою логікою валідації

## Acceptance Criteria

- [x] Редагування тікета використовує `in-out property <TicketStr> ticket` замість `editing_*` properties
- [x] Two-way model bindings коректно синхронізують зміни між UI та моделлю
- [x] Валідація полів працює без збоїв
- [x] Всі існуючі тести проходять (91 тест, 0 помилок)

## Resolution

### Підсумок

Замінено 6 окремих `editing_*` properties на єдиний `in-out property <TicketStr> editing_ticket` з двонаправленим зв'язуванням полів.

### Зміни в коді
| Файл | Зміна |
|---|---|
| `ui/app.slint:83-86` | Замінено 6 `editing_*` властивостей на `in-out property <TicketStr> editing_ticket` + повернуто `show_ticket_edit_dialog` |
| `ui/app.slint:70-79` | Додано callback `test-trigger-edit-ticket(string, string, string, string, int)` для тестування |
| `ui/app.slint:362-377` | Оновлено handler `test-trigger-add-ticket` — використовує індивідуальні присвоювання полів struct |
| `ui/app.slint:404-418` | Додано handler `test-trigger-edit-ticket` — створює TicketStr з параметрів |
| `ui/app.slint:403-421` | Відновлено пропущений блок `if (show_ticket_view_dialog): TicketView` |
| `ui/dialogs/ticket_edit.slint:1-11` | Замінено scalar властивості на `in-out property <TicketStr> ticket`; додано import `TicketStr` |
| `ui/dialogs/ticket_edit.slint:52-68` | Оновлено binding на `ticket.id`, `ticket.title`, тощо |
| `ui/dialogs/ticket_edit.slint:124-170` | Оновлено Points ComboBox — працює з `ticket.points` |

### Додані тести
- Немає нових тестів — оновлено існуючі (`run_gui_interaction_cycle`, `test_gui_user_desync`) для використання нового API

### Побічний фікс
- `ui/app.slint:403-421` — відновлено пропущений блок `if (show_ticket_view_dialog): TicketView { ... }` який був випадково видалений під час редагування

### Оновлена документація
Не потрібно — структурні зміни не впливають на зовнішній API