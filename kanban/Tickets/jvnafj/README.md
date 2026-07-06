---
title: "Upgrade from Slint 1.15 to 1.17"
created_at: 2026-07-03 09:24:20
assigned_to: "admin"
author: "user"
points: 8
attachment_count: 0
---
## Контекст

Проєкт використовує slint = "1.15". Випущено дві нові версії:
- Slint 1.16 (квітень 2026): KeyBinding, StyledText + @markdown, multi-touch gestures
- Slint 1.17 (червень 2026): Native Drag & Drop, SystemTrayIcon, Tooltips, RadioGroup, two-way model bindings

Поточна реалізація використовує ручні патерни, які можна замінити на нативні елементи.

## Поточний стан коду

| Компонент | Статус | File |
|---|---|---|
| Drag & Drop | Ручний (TouchArea + 3-level callback chain) | ui/components/ticket_card.slint, ui/app.slint#363-385 |
| Гарячі клавіші | Ручний key-pressed(event) з українською розкладкою | ui/app.slint#159-211 |
| Tooltips | Custom floating Rectangle з 4 property | ui/app.slint#76-80, 612-631 |
| Ticket editing | 6 manual editing_* properties + state copy | ui/app.slint#94-101, ui/dialogs/ticket_edit.slint |
| Description rendering | Plain Text (без markdown) | ui/dialogs/ticket_view.slint#153-158 |

## План покращень (пріоритезовано)

Є приклад маленької дошки kanban з drang and drop: https://raw.githubusercontent.com/slint-ui/slint/refs/heads/master/examples/dnd-kanban/main.rs https://raw.githubusercontent.com/slint-ui/slint/refs/heads/master/examples/dnd-kanban/kanban.slint

### P0 - Критичні

1. Native Drag & Drop (DragArea / DropArea)
   - Замінити TouchArea-based drag на нативний DnD
   - Прибирає ~400 рядків callback-коду, замінюється на ~200
   - Прибирає is_dragging, mouse_x, mouse_y, ghost-Rectangle
   - Потрібен Rust-side глобальний Api з make-transfer/read-transfer callback-ами
   - Підзадача: #646xe0

2. KeyBinding - заміна ручного key-pressed(event)
   - Декларативні @keys(Control+F), @keys(Control+N), @keys(Control+M), @keys(Escape)
   - Прибирає ручну обробку української розкладки
   - Підзадача: #dof4c9

### P1 - Важливі

3. Native Tooltips - заміна custom Rectangle tooltip
   - Вбудований Tooltip { text: @markdown("...") } елемент
   - Прибирає tooltip_text, tooltip_visible, tooltip_x, tooltip_y
   - Підзадача: #10i9d6

4. Two-way model bindings on rows - text <=> item.name
   - Прибирає 6 editing_* properties, замінюється на in property <TicketStr> ticket + двонаправлене зв'язування
   - Потрібна перевірка сумісності (in property vs model)
   - Підзадача: #gykrr4

### P2 - Бажані

5. StyledText + @markdown для рендерингу описів тікетів
   - Рендеринг markdown-тексту в описі тікета
   - Підзадача: #gwb1dm

6. SystemTrayIcon - minimize-to-tray функціональність
   - Іконка в системному треї з контекстним меню
   - Підзадача: #embarr

7. Cross-axis alignment - покращення layout
   - Використання align-items / justify-content замість ручного позиціонування
   - Підзадача: #k8d6p4

## Ризики

- Two-way bindings на in property (не model) можуть не працювати
- StyledText має обмежений набір форматів (не повний HTML)

## Acceptance Criteria

- [ ] Проєкт компілюється з slint = "1.17" без помилок
- [ ] Drag & Drop карток між колонками працює нативно (DragArea/DropArea)
- [ ] Гарячі клавіші (Ctrl+F, Ctrl+N, Ctrl+M, Escape) працюють через KeyBinding
- [ ] Tooltips на картках та посиланнях працюють як вбудований елемент
- [ ] Всі існуючі тести проходять
- [ ] Функціональність GUI не деградувала порівняно з 1.15

---
Sources: https://slint.dev/blog/slint-1.16-released, https://slint.dev/blog/slint-1.17-released