---
title: "P0: Замінити ручні гарячі клавіші на KeyBinding (Slint 1.17)"
created_at: 2026-07-04 14:54:55
assigned_to: "admin"
author: "user"
points: 2
attachment_count: 0
---
## Контекст

Зараз гарячі клавіші реалізовані вручну через `key-pressed(event)` з обробкою української розкладки у `ui/app.slint#159-211`. Це ручна парса подій, яка залежить від конкретної розкладки.

Slint 1.17 додає `KeyBinding` з `@keys(Control+F)`, `@keys(Control+N)`, `@keys(Control+M)`, `@keys(Escape)` — декларативні скорочення, які працюють незалежно від розкладки.

## Очікувана поведінка

1. Замінити ручну обробку `key-pressed` на декларативні `KeyBinding`
2. Підтримувати: Ctrl+F (пошук), Ctrl+N (новий тікет), Ctrl+M (мій фільтр), Escape (закрити)
3. Прибрати ручну обробку української розкладки

## Ризики

- `KeyBinding` може конфліктувати з існуючими обработчиками подій
- Потрібно перевірити сумісність з мобільними пристроями (де клавіатури немає)

## Acceptance Criteria

- [ ] Ctrl+F відкриває пошук через `KeyBinding @keys(Control+F)`
- [ ] Ctrl+N створює новий тікет через `KeyBinding @keys(Control+N)`
- [ ] Ctrl+M вмикає фільтр "мої тікети" через `KeyBinding @keys(Control+M)`
- [ ] Escape закриває поточний діалог через `KeyBinding @keys(Escape)`
- [ ] Відсутній ручний key-pressed код
- [ ] Всі існуючі тести проходять
