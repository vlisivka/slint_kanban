---
title: Помилка розбору YAML (виправлено)
created_at: 2026-06-21 14:24:49
assigned_to: "user"
author: "user"
points: 0
attachment_count: 0
---
Опис помилки:

коли у файлі Gettext .po зустрічається видалене повідомлення, наприклад:

```
#~ msgid "Foo"
#~ msgstr "Bar"
```

То воно повинно зберігатися як чистий коментар, без msgid та msgst, та не приймати участі у перекладах чи ревʼю.

Актуальна поведінка:

Коментарі перекладаються як повідомлення.

Очікувана поведінка:

Коментарі переносяться у вихідний файл як коментарі, без обробки.

```

## Resolution

**Проблема**: `Ticket::save()` писав `title` без лапок у YAML фронтматері. Якщо title містить двокрапку (наприклад "Error: colon in title"), serde_yaml інтерпретує її як роздільник ключів → "mapping values are not allowed in this context".

**Фікс**: `src/model/ticket.rs:288` — загортувати `title` у дужки: `title: "{}"` замість `title: {}`.

**Перевірка**: Додано тест `test_ticket_save_load_with_colon_in_title` (RED→GREEN). Усі 55 тестів проходять.

**Додатково**: Створено `docs/memory/yaml-frontmatter-quoting.md` та керований навик `yaml-frontmatter-quoting` для запобігання цьому багу в майбутньому.