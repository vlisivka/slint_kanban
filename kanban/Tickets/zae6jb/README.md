---
title: "Bug: GUI не оновлюється після Drag & Drop переносу тікету між чергами (stale state)"
created_at: 2026-07-03 10:00:00
updated_at: 2026-07-03 22:56:40
assigned_to: "admin"
author: "user"
points: 1
attachment_count: 0
---
# GUI не оновлюється після Drag & Drop переносу тікету між чергами (stale state)

## Контекст

При перенесенні тікету з однієї черги в іншу на дошці через drag-and-drop (мишкою), тікет вірно фізично переміщується на диску, але GUI показує старий стан — копія тікета залишається у вихідній черзі. При спробі перенести «залишок» знову — виникає помилка: тікет не існує в цій черзі (бо він вже реально переміщений).

## Репродукція

1. Відкрити GUI (`./kanban.sh open`)
2. Перетягнути тікет мишкою з однієї колонки в іншу
3. **Факт**: тікет переміщено на диску (symlink перенесено)
4. **Баг**: у GUI тікет все ще відображається у вихідній колонці
5. Спроба перетягнути «залишок» → помилка: «Ticket X not found in queue Y»

Лог підтверджує, що watcher бачить зміну на диску:
```
Controller: Moving 4fy8ug from 3.SprintBacklog to 2.ProductBacklog
[WATCHER] Виявлено зміни, запуск перевантаження...
```

## Аналіз проблеми

### Різнорідність обробки action handlers

У `src/controller.rs` всі action-хендлери після успішної операції викликають `self.reload()`, КРІМ `handle_move_ticket`:

| Handler | Викликає reload()? |
|---|---|
| `handle_delete_ticket` (l.381) | ❌ НІ — покладається на watcher |
| `handle_create_ticket` (l.402) | ❌ НІ — покладається на watcher |
| `handle_update_ticket` (l.429) | ❌ НІ — покладається на watcher |
| `handle_move_ticket` (l.319) | ❌ НІ — покладається на watcher |
| `handle_add_user` (l.560) | ✅ ТАК |
| `handle_toggle_show_only_mine` (l.571) | ✅ ТАК |
| `handle_toggle_manage_only_mine` (l.580) | ✅ ТАК |
| `handle_set_queue_limit` (l.604) | ✅ ТАК |
| `handle_rename_queue` (l.756) | ✅ ТАК |
| `handle_delete_queue` (l.760) | ✅ ТАК |

### Чому watcher не спрацьовує?

`board.move_ticket()` викликає `std::fs::rename(source_link, dest_link)` — це rename symlink across directories.

Watcher події (main.rs:66-73):
```rust
let should_reload = match event.kind {
    EventKind::Create(_) | EventKind::Remove(_) => true,
    EventKind::Modify(m) => matches!(
        m,
        notify::event::ModifyKind::Data(_) | notify::event::ModifyKind::Name(_)
    ),
    _ => false,
};
```

На Linux inotify генерує `IN_MOVED_FROM` + `IN_MOVED_TO` при rename. `notify` мапить їх на `EventKind::Modify(ModifyKind::Name(...))`.

Але є нюанс: `std::fs::rename()` на symlink — це операція над **самою лінкою**, а не над файлом, на який вона вказує. inotify може не фіксувати зміни імені для symlink-об'єктів у деяких конфігураціях.

### Ймовірні причини (пріоритезовано)

1. **Найімовірніше**: `handle_move_ticket` не викликає `self.reload()` — покладається на watcher, але watcher або не бачить symlink rename, або є race condition між rename і watch event
2. **Можливо**: `notify` на Linux з inotify не коректно фіксує rename symlink (відомий edge-case)
3. **Можливо**: debounce у watcher (500ms) створює race condition — подія приходить, але debounce ще не завершився

## План виправлення

### Фаза 1: Репродукція + тест (TDD Red-Green-Refactor)

**Завдання 1.1**: Створити тест, що репродуциє баг
- Використовувати `i-slint-backend-testing` для headless GUI тестування
- Симулювати: створити 2 черги з тікетами → натиснути кнопку/подію яка імітує drag-drop
- Перевірити: після move-ticket callback, `board_queues` оновлено коректно

**Завдання 1.2**: Зафіксувати баг у тесті (RED phase — тест падає)

**Завдання 1.3**: Виправити баг (GREEN phase — тест проходить)

### Фаза 2: Виправлення

**Гіпотеза А (найпростіша)**: додати `self.reload()` у `handle_move_ticket` після успішного move
```rust
if let Err(e) = board.move_ticket(&ticket_id, &source_id, &resolved_target_id) {
    self.show_error(&e.to_string());
} else {
    let _ = self.reload();  // ← додати
}
```

### Фаза 3: AAR (After Action Review)

Аналіз чому існуючі тести не виявили проблему:
- Перевірити наявність GUI-тестів для drag-and-drop сценаріїв
- Оцінити покриття `controller.rs` тестами
- Запропонувати покращення: CI-run GUI-тестів, integration tests з `i-slint-backend-testing`

## Acceptance Criteria

- [x] Існує тест, що репродукує баг (RED phase)
- [x] Тест проходить після виправлення (GREEN phase)
- [x] Після drag-and-drop GUI показує актуальний стан (тікет в новій черзі, не у старій)
- [x] Подвійне переміщення тікету не дає помилки «not found in queue»

## Sources

- `src/controller.rs:319-348` — handle_move_ticket (не викликає reload)
- `src/main.rs:66-73` — watcher event filtering
- `src/model/board.rs:636-696` — move_ticket (std::fs::rename на symlink)
- `src/controller.rs:381-400` — handle_delete_ticket (також не викликає reload, але може спрацьовувати watcher)
## Resolution comments

### Підсумок

Виправлено баг, при якому GUI не оновлювався після drag-and-drop переносу тікету між чергами. Було виявлено та виправлено дві проблеми:

1. `handle_move_ticket` не викликав `self.reload()` після успішного переміщення (відповідно до гіпотези з опису багу)
2. `sync_board_data` не видаляв старі рядки з VecModel при зменшенні кількості тікетів у черзі

### Зміни в коді
| Файл | Зміна |
|---|---|
| `src/controller.rs:345-349` | Додано `let _ = self.reload();` у `handle_move_ticket` після успішного move |
| `src/controller.rs:172-177` | Додано логіку `else if new_len < current_len` для видалення зайвих рядків з tickets_model у `sync_board_data` |
| `ui/app.slint:75,448-451` | Додано `test-trigger-move-ticket` callback для тестування drag-and-drop |

### Побічний фікс
- Виявлено, що `can_manage_ticket` блокував переміщення тікетів у тестах через конфлікт `active_user` ("user") з `assigned_to` ("user1"). Тест налаштований на `manage_only_mine = false`.

### Додані тести
- `test_gui_move_ticket_updates_board()` — перевірка що GUI оновлюється після move-ticket без ручного reload()

### Оновлена документація
Не оновлено — зміни стосуються лише виправлення багів у існуючих функціях.