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

- [ ] Tooltips на картках тікетів працюють через вбудований `Tooltip` елемент
- [ ] Tooltips на посиланнях на тікети теж використовують `Tooltip`
- [ ] Текст tooltip рендериться з markdown через `@markdown`
- [ ] Відсутній ручний tooltip Rectangle код
- [ ] Всі існуючі тести проходять
