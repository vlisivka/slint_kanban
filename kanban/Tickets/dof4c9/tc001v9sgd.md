---
author: user
created_at: 2026-07-05 22:06:28
updated_at: 2026-07-05 22:06:28
---
# План реалізації

## Контекст
`FocusScope` у `app.slint` має ручний обробник `key-pressed(event)` (lines 149-201), який перевіряє `event.modifiers.control` та `event.text` для визначення комбінацій. Також потрібно врахувати українську розкладку (а/А, т/Т, ь/Ь).

## Зміни

### `ui/app.slint`
1. Знайти `FocusScope` в app.slint
2. Замінити `key-pressed(event) => { ... }` на `KeyBinding` елементи:
   - `@keys(Control + F)` → `shortcut-open-new-ticket-dialog()` (пошук)
   - `@keys(Control + N)` → `shortcut-open-new-ticket-dialog()` (новий тікет)
   - `@keys(Control + M)` → `UserGlobal.toggle-show-only-mine(!UserGlobal.show-only-mine)`
   - `@keys(Escape)` → закрити всі діалоги/пошук
3. Прибрати ручну обробку української розкладки (перевірка "а", "т", "ь")

## Перевірка
- `cargo test` — всі тести проходять
