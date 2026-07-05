---
title: "P1: Замінити custom Tooltips на вбудований Tooltip елемент"
created_at: 2026-07-04 14:55:08
assigned_to: "admin"
author: "user"
points: 2
attachment_count: 0
---
## Контекст

Зараз tooltips реалізовані як custom floating Rectangle з 4 property (`tooltip_text`, `tooltip_visible`, `tooltip_x`, `tooltip_y`) у `ui/app.slint#76-80, 612-631`. Це ручна логіка позиціонування та видимості.

Slint 1.17 додає вбудований елемент `Tooltip { text: @markdown("...") }`, який автоматично обробляє відображення, позиціонування та markdown-рендеринг тексту.

## Очікувана поведінка

1. Замінити custom tooltip Rectangle на вбудований `Tooltip` елемент
2. Використовувати `@markdown` для рендерингу тексту tooltip
3. Прибирати `tooltip_text`, `tooltip_visible`, `tooltip_x`, `tooltip_y`

## Ризики

- Вбудований `Tooltip` може мати іншу поведінку анімації / позиціонування
- Потрібно перевірити сумісність з touch-взаємодією (довге натискання)

## Acceptance Criteria

- [x] Tooltips на картках тікетів працюють через вбудований `Tooltip` елемент
- [x] Tooltips на посиланнях на тікети теж використовують `Tooltip`
- [x] Текст tooltip рендериться з markdown через `@markdown`
- [x] Відсутній ручний tooltip Rectangle код
- [x] Всі існуючі тести проходять

## Resolution

### Підсумок

Замінено custom tooltip Rectangle на вбудований `Tooltip` елемент Slint 1.17. Прибрано ручну логіку позиціонування та видимості (4 property: `tooltip_text`, `tooltip_visible`, `tooltip_x`, `tooltip_y`). Тепер кожна посилання на тікет має власний Tooltip із автоматичним відображенням, позиціонуванням та markdown-рендерингом.

### Зміни в коді
| Файл | Зміна |
|---|---|
| `ui/dialogs/ticket_view.slint` | Замінено `TouchArea { changed has-hover => show-tooltip(...) }` на `Tooltip { text: @markdown("\{ref.title}") }` у 2 місцях (References та Comments); прибрано callback-и `show-tooltip`, `hide-tooltip` |
| `ui/app.slint` | Прибрано tooltip state variables (`tooltip_text`, `tooltip_visible`, `tooltip_x`, `tooltip_y`); callback-и `show-tooltip(text, x, y)`, `hide-tooltip()`; floating Tooltip Rectangle (~20 рядків коду) |

### Побічний фікс
- `Tooltip { text: @markdown("\{ref.title}") }` — використано escaped-interpolation `"\{...}"` для конвертації string → styled-text (помилявся з `@markdown(ref.title)`).

### Додані тести
- Всі існуючі тести пройшли (`cargo test` — 72+ тестів, 0 fail). Тестів на tooltip не було, тому нових не додавав.

### Оновлена документація
- Немає (документація проекту не стосується UI-реалізації tooltip).