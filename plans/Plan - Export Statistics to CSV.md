# План реалізації - Експорт статистики в CSV

Цей план описує додавання можливості експорту статистики борди у формат CSV через CLI.

## Мета
Додати параметр `--csv` до команди `stats`, щоб користувач міг отримати дані у машиночитаному форматі для подальшого аналізу (наприклад, в Excel або через скрипти).

## Зміни

### 1. CLI (`src/cli.rs`)
- Додати поле `csv: bool` до варіанту `Stats` переліку `Commands`:
  ```rust
  Stats {
      /// Filter by user
      #[arg(long)]
      user: Option<String>,

      /// Export in CSV format
      #[arg(long)]
      csv: bool,
  }
  ```

### 2. Main (`src/main.rs`)
- В обробнику `Commands::Stats` додати перевірку:
  - Якщо `csv == true`, викликати нову функцію `print_stats_csv(summary, out)`.
  - Інакше використовувати існуючий `print_stats_human_readable`.
- Реалізувати функцію `print_stats_csv`. Формат CSV буде включати всі секції статистики (Summary, Queues, Users, Trends, Burndown) з колонкою `Type` для розрізнення рядків.

#### Формат CSV:
Колонки: `Type`, `Category/Name`, `Metric`, `Value`, `Unit`
Приклад:
- `Summary,General,Total Tickets,10,count`
- `Queue,1.Incoming,Count,5,tickets`
- `User,alice,Count,3,tickets`
- `Trend,2024-02-10,Total Points,50,pts`
- `Burndown,2024-02-10,Remaining Points,40,pts`

### 3. Тести
- Додати інтеграційний тест у `tests/cli_tests.rs` (або аналогічний), який запускає `slint_kanban stats --csv` та перевіряє:
  - Наявність заголовків.
  - Наявність ключових слів (Summary, Queue, User).
  - Що вихід є валідним CSV (хоча б базово).

## Ручна перевірка
1. Запустити `cargo run -- stats --csv`.
2. Перевірити, що вивід містить рядки, розділені комами.
3. Спробувати перенаправити вивід у файл: `cargo run -- stats --csv > stats.csv`.
4. Відкрити файл у текстовому редакторі або табличному процесорі.
