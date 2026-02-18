# Plan - Search functionality

## Meta
- **ПОТОЧНЕ_ЗАВДАННЯ**: Implement search functionality (Search and Filter).

## Мета
Додати можливість повнотекстового пошуку за заголовками та описами тікетів.

## Зміни

### [Component Name] UI

#### [MODIFY] [app.slint](file:///home/vlisivka/workspace/slint_kanban/ui/app.slint)
- Додати `search_query: string` властивість до `App`.
- Додати callback `search_edited(string)` до `App`.
- Переробити структуру `App`: додати `VerticalBox` як кореневий елемент, щоб розмістити пошуковий рядок зверху.
- Додати `LineEdit` для пошуку в хедер.
- Додати кнопку для очистки поля пошуку.

### [Component Name] Logic

#### [MODIFY] [model.rs](file:///home/vlisivka/workspace/slint_kanban/src/model.rs)
- Додати метод до `Ticket` для перевірки відповідності пошуковому запиту (регістронезалежно).

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- Зберегти поточний стан пошукового запиту.
- Оновити `sync_ui_with_board`, щоб він враховував фільтр.
- Реалізувати callback `on_search_edited` в Rust, який оновлює стан та викликає перемальовування.

## Тести
### Автоматичні тести
- Додати тест в `model/tests.rs` для перевірки логіки фільтрації `Ticket`.
- Додати тест в `main_tests.rs`, який перевіряє, що після зміни запиту в UI модель містить лише відфільтровані тікети.

## Ручна перевірка
1. Запустити додаток.
2. Створити кілька тікетів з різними заголовками (наприклад, "Apple", "Banana", "Cherry").
3. Ввести "an" у поле пошуку.
4. Переконатися, що відображаються лише тікети "Banana" (або інші, що містять "an").
5. Стерти пошук і переконатися, що всі тікети повернулися.
