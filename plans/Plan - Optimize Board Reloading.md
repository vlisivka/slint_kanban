# Plan - Optimize Board Reloading

## Meta
- **ПОТОЧНЕ_ЗАВДАННЯ**: Fix double reload and optimize ticket loading.

## Аналіз проблеми
1.  **Подвійне перезавантаження**: При перемиканні видимості черги `on_toggle_queue_visibility` викликає `reload_board` вручну, а потім `watcher` бачить зміну `config.toml` і викликає `reload_board` вдруге.
2.  **Зайве навантаження**: `Board::load` завжди завантажує всі тікети з усіх черг, навіть якщо вони приховані.

## Пропоновані зміни

### [Component Name] Logic (model.rs)

#### [MODIFY] [model.rs](file:///home/vlisivka/workspace/slint_kanban/src/model.rs)
- Оновити `load_queue`, щоб вона завантажувала тікети тільки якщо черга видима (`visible == true`).
- Це дозволить прискорити `Board::load` при приховуванні черг.

### [Component Name] Watcher (main.rs)

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- Змінити дебаунс з "ковзного вікна" на "фіксоване вікно". Замість `while rx.recv_timeout(...).is_ok()` використовувати фіксований сон та `try_recv()`. Це гарантує, що ми не будемо чекати вічно, якщо події приходять часто.
- Переконатися, що вочер не створює зайвих подій при читанні.

### [Component Name] Sync (main.rs)

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- В `on_toggle_queue_visibility`:
    - Замінити `Board::load` на `Config::load` (дешевше).
    - Видалити прямий виклик `reload_board`. Нехай `watcher` обробляє оновлення UI після запису `config.toml`.
    - (Опціонально) Оновити властивість `visible` у Slint моделі напряму для миттєвої реакції, але обережно, щоб не було конфлікту з наступним перезавантаженням від вочера. Краще просто покластися на вочера, якщо він працює стабільно.

## Тести
### Автоматичні тести
- `cargo test` має проходити без помилок.

## Ручна перевірка
1. Запустити додаток.
2. Перемкнути видимість черги.
3. Перевірити консоль: має бути лише один запис "Reloading board #...".
4. Перевірити, що прихована черга зникає, а при повторному показі — з'являється з тікетами.
