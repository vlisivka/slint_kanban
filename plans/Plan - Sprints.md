# План реалізації: Спринти та гнучкі воркфлови (Фаза 1.5)

## Мета
Додати підтримку спринтів (CLI керування, відображення у GUI) та гнучке налаштування воркфловів (`start_queues`, `done_queues`). Це є передумовою для обчислення часових метрик (Time tracking, Cycle Time, Throughput) у наступних фазах. 

## Зміни (Архітектура та Код)

1. **Конфігураційні структури (`src/model/config.rs`)**:
   - Створити структуру `Sprint`:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize)]
     pub struct Sprint {
         pub number: u32,
         pub name: String,
         pub start_date: String,
         pub end_date: String,
     }
     ```
   - Створити структуру `Workflow`:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize, Default)]
     pub struct Workflow {
         pub start_queues: Vec<String>,
         pub done_queues: Vec<String>,
     }
     ```
   - Розширити `KanbanConfig`:
     - Додати поле `pub sprints: Vec<Sprint>`. По дефолту — пустий масив.
     - Додати поле `pub workflows: HashMap<String, Workflow>`.

2. **Керування спринтами (`src/model/board.rs` або `src/model/config.rs`)**:
   - Додати метод до `KanbanConfig` або `Config` для отримання "поточного спринта" (`get_current_sprint(&self) -> Option<&Sprint>`), який перевірятиме `chrono::Local::now().naive_local().date()` проти `start_date` та `end_date`.
   - Рефакторинг керування спринтами: CLI оброблюватиме логіку додавання/видалення/отримання списку спринтів.

3. **CLI-команда `sprint` (`src/cli.rs` та `src/main.rs`)**:
   - В `src/cli.rs` додати новий варіант `Sprint` до `enum Commands` із відповідними підкомандами (`List`, `Current`, `Add`, `Update`, `Remove`), як описано в `research.md`.
   - В `src/main.rs` додати обробник для `Commands::Sprint`. Цей блок виконуватиме:
     - Отримання поточних спринтів: `board.config.kanban.sprints`
     - Оновлення: пошук за `number` і зміна атрибутів
     - Додавання: перевірка на дублікати `number`
     - Запис нової конфігурації у диск за допомогою `board.config.write(&root_path)`.

4. **Відображення у GUI (`ui/app.slint` та `src/main.rs`)**:
   - В `ui/common.slint` створити `SprintStr`:
     ```slint
     export struct SprintStr {
         number: int,
         name: string,
     }
     ```
   - В `ui/app.slint` додати властивість:
     ```slint
     in-out property <bool> has_active_sprint: false;
     in-out property <SprintStr> active_sprint;
     ```
   - У `ui/app.slint` в `HorizontalLayout` (де знаходяться кнопки Board Info та Statistics) додати Text компонент (або Badge), який показуватиме "🏃 Sprint {number}: {name}", видимий тільки коли `has_active_sprint` == true.
   - В `src/controller.rs` під час ініціалізації (`sync_config` або `sync_board_data`) обчислювати поточний спринт через `board.config.get_current_sprint()` і передавати його в UI.

5. **Оновлення `slint_kanban stats` (`src/cli.rs` та `src/main.rs`)**:
   - Додати прапорець `--sprint <number>` до команди `stats`. (Поки що він працюватиме на рівні CLI парсингу, а логіка фільтрації тікетів в межах дат спринта буде додана у Фазі 3, або ж ми можемо вже зараз змінити `get_board_summary` так, щоб він фільтрував тікети за датою з урахуванням спринта, якщо це можливо/бажано). *Рішення: поки що просто додати прапорець, повноцінно він буде працювати з log-based метриками.*

## Тести

1. **Unit tests (`src/model/tests/config_tests.rs`)**:
   - Додати тести розбору/звертання (serialization) для структур `Sprint` та `Workflow`.
   - Додати тест `test_get_current_sprint`: створити 3 спринти (минулий, поточний, майбутній) і перевірити, чи правильно `get_current_sprint` повертає поточний відповідно до поточної дати (або мокнути дату, або створити спринти з датами +- відносно сьогоднішнього дня).

2. **CLI tests (`src/main_tests.rs`)**:
   - Додати `test_cli_sprint_crud`:
     - Додати спринт через CLI `sprint add`.
     - Зробити `sprint list` і перевірити наявність.
     - Переконатися, що `sprint update` працює.
     - Перевірити `sprint remove`.

3. **GUI tests (`src/gui_tests.rs`)**:
   - Тестувати, що візуальний індикатор поточного спринта встановлюється/не встановлюється залежно від ініціалізації конфігурації.

## Ручна перевірка

1. Зібрати та виконати: `cargo run -- sprint add --number 1 --name "Test Sprint" --start <вчора> --end <завтра>`.
2. Перевірити, що `cargo run -- sprint list` показує цей спринт.
3. Запустити `cargo run` (GUI) і пересвідчитись, що у тулбарі відображається "🏃 Sprint 1: Test Sprint".
4. Відкрити `Kanban/config.toml` та перевірити наявність секцій `[[sprints]]` та `[workflows]`.

---
**Результат:** Фаза 1.5 реалізована успішно. Додано підтримку спринтів у модель, CLI та GUI. Налаштовано гнучкі воркфлови у конфігурації. Тести проходять (unit + CLI integration). GUI відображає поточний спринт у тулбарі.
