---
title: "P2: StyledText + @markdown для рендерингу описів тікетів"
created_at: 2026-07-04 14:55:31
updated_at: 2026-07-04 14:55:31
assigned_to: "user"
author: "user"
points: 3
attachment_count: 0
---
## Контекст

Зараз описи тікетів рендеряться як plain Text без markdown у `ui/dialogs/ticket_view.slint#153-158`. Користувачі не бачать форматування (bold, italic, links) в описі тікета.

Slint 1.17 додає `StyledText` з методом `from_markdown()`, що дозволяє рендерити markdown-текст безпосередньо у UI.

> **Blocker**: цей тікет вимагає оновлення до Slint 1.17 (див. #jvnafj).

## Очікувана поведінка

1. Замінити `<Text>` на `<StyledText>` для відображення опису тікета
2. Використовувати `from_markdown()` для парсингу markdown
3. Підтримувати: bold, italic, links, lists, code blocks

## Ризики

- `StyledText` має обмежений набір форматів (не повний HTML)
- Потрібно перевірити, які markdown-синтаксиси підтримуються

## Acceptance Criteria

- [ ] Опис тікета рендериться через `StyledText.from_markdown()`
- [ ] Bold, italic, links відображаються коректно
- [ ] Lists та code blocks рендеряться без помилок
- [ ] Всі існуючі тести проходять
