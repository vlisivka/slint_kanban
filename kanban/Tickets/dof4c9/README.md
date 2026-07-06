---
title: "P0: Замінити ручні гарячі клавіші на KeyBinding (Slint 1.17)"
created_at: 2026-07-04 14:54:55
assigned_to: "user"
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

- `KeyBinding` може конфліктувати з існуючими обробниками подій

## Acceptance Criteria

- [x] Ctrl+F відкриває пошук через `KeyBinding @keys(Control+F)`
- [x] Ctrl+N створює новий тікет через `KeyBinding @keys(Control+N)`
- [x] Ctrl+M вмикає фільтр "мої тікети" через `KeyBinding @keys(Control+M)`
- [x] Escape закриває поточний діалог через `KeyBinding @keys(Escape)`
- [x] Відсутній ручний key-pressed код
- [x] Всі існуючі тести проходять

## Resolution

### Підсумок

Замінено ручну обробку гарячих клавіш через `key-pressed(event)` на декларативні `KeyBinding` елементи Slint 1.17. Прибрано ручний парсинг подій та обробку української розкладки (перевірка "а"/"А", "т"/"Т", "ь"/"Ь").

### Зміни в коді
| Файл | Зміна |
|---|---|
| `ui/app.slint` | Прибрано `key-pressed(event)` handler (~50 рядків); додано 4 `KeyBinding` елементи з `@keys(Control+F)`, `@keys(Control+N)`, `@keys(Control+M)`, `@keys(Escape)` |

### Побічний фікс
- Немає.

### Додані тести
- Всі існуючі тести пройшли (`cargo test` — 72+ тестів, 0 fail). Тестів на keybinding не було.

### Оновлена документація
- Немає (документація проекту не стосується UI-реалізації hotkeys).