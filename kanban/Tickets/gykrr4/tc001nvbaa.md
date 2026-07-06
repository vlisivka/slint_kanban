---
author: user
created_at: 2026-07-06 10:10:41
updated_at: 2026-07-06 10:10:41
---
# План виконання: Two-way model bindings замість editing_* properties

## Завдання
Замінити 6 `editing_*` property та ручну копію стану на єдиний `in-out property <TicketStr> ticket` з дво-спрямованим зв'язуванням.

## Зміни

### 1. `ui/app.slint` — заміна стану редагування
Замінити:
```slint
editing_ticket_id, editing_ticket_title, editing_ticket_description,
editing_ticket_assignee, editing_ticket_author, editing_ticket_points
```
на:
```slint
in-out property <TicketStr> editing_ticket;
```

Оновити:
- `shortcut-open-new-ticket-dialog` — копіювати стан у `editing_ticket`
- `edit-ticket(active_ticket)` — копіювати `active_ticket` у `editing_ticket`
- Передавати `editing_ticket` до `TicketEdit` замість окремих полів

### 2. `ui/dialogs/ticket_edit.slint` — замінити scalar властивості на struct
Замінити:
```slint
in-out property <string> title_text;
in-out property <string> description_text;
in-out property <string> assigned_to;
in property <string> author;
in property <string> ticket_id;
in-out property <int> points;
```
на:
```slint
in-out property <TicketStr> ticket;
```

Оновити bindings:
- `text <=> ticket.title` (замість `text <=> title_text`)
- `text <=> ticket.description` (замість `text <=> description_text`)
- `current-value <=> ticket.assigned_to` (для ComboBox)
- `selected(val) => { ticket.points = ... }` (для Points ComboBox)

### 3. Тестування
- Перевірити що існуючі тести проходять
- Додати регресійний тест на двонаправлене зв'язування

## Ризики
- `in-out property <TicketStr>` може не підтримувати двонаправлене зв'язування на level of struct fields (потрібно перевірити Slint 1.17)
