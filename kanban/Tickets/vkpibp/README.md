---
title: Помилка розбору YAML (виправлено)
created_at: 2026-06-21 14:24:49
updated_at: 2026-06-21 15:00:00
assigned_to: ""
author: "user"
points: 0
attachment_count: 0
---
```
Project root: /home/vlisivka/workspace/po-tools-rust
Використовується корінь Канбан: /home/vlisivka/workspace/po-tools-rust/kanban
Warning: Failed to load ticket at "/home/vlisivka/workspace/po-tools-rust/kanban/Tickets/llnz65": Не вдалося розібрати YAML в /home/vlisivka/workspace/po-tools-rust/kanban/Tickets/llnz65/README.md: mapping values are not allowed in this context at line 1 column 15
Controller: Updating users list in UI...
```

Текст тікета:

```
---
title: Помилка: блок коментарів (#~) перекладається як повідомлення з пустим msgid.
created_at: 2026-06-21 14:20:21
updated_at: 2026-06-21 14:20:21
assigned_to: ""
author: "user"
points: 1
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