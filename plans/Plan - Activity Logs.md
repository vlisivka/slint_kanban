# Plan - Activity Logs

## Мета
Реалізувати систему журналювання дій користувачів у децентралізованій Kanban-дошці (додаток Slint Kanban) у форматі Markdown-таблиці з JSON-аргументами у кінці кожного рядка. Це дозволить в майбутньому легко будувати статистику, зберігаючи людську читабельність логів та уникаючи конфліктів при синхронізації між пристроями/користувачами.

## Зміни

### 1. `config.rs` (User Settings)
- **Зміни у `UserConfig`**: Додати поле `machine_id: Option<String>` (може бути пустим у старих конфігах).
- **Логіка**: При завантаженні `UserConfig`, якщо `machine_id` відсутній або порожній, згенерувати випадковий короткий ID (додати крейт `nanoid` або використовувати `uuid`, або написати простий генератор випадкового рядка, щоб не тягнути нову залежність, наприклад, 6 випадкових літер/цифр).
- Після генерації зберегти конфіг.

### 2. `model.rs` (Опис дії)
- **Створення `ActionPayload` Enum**:
  Створити перелічення (Enum) `ActionPayload` з `#[derive(Serialize, Deserialize)]` зі всіма можливими діями, наприклад:
  ```rust
  #[derive(Serialize, Deserialize, Debug, Clone)]
  #[serde(tag = "action")]
  pub enum ActionPayload {
      CreateTicket { id: String, title: String },
      UpdateTicket { id: String },
      ChangeStatus { id: String, from: String, to: String },
      AddComment { id: String, comment_id: String },
      AssignTicket { id: String, assignee: Option<String> },
      AttachFile { id: String, filename: String },
  }
  ```
  Використання `#[serde(tag = "action")]` змушує `serde_json` автоматично додавати поле `"action"` під час серіалізації.

- **Створення функції журналування**:
  Створити функцію `Board::append_log_entry(&self, payload: ActionPayload, description: &str)` або як окрему допоміжну функцію.
  - Шлях до файлу журналу: `Kanban/logs/log_{active_user}_{machine_id}.md`.
  - Перевірити наявність файлу. Якщо його немає:
    - Створити файл з заголовками:
      ```markdown
      # User Activity Log: {username}
      
      | **Date** | **Action** | **Action description** | **JSON** |
      | :--- | :--- | :--- | :--- |
      ```
  - Отримати поточний час у форматі ISO 8601 без мілісекунд (`chrono::Local::now().to_rfc3339_opts(...)`).
  - Отримати назву дії англійською мовою (наприклад, "CREATE_TICKET"). Можна реалізувати `Display` для `ActionPayload` або витягувати значення тегу з `serde`.
  - Серіалізувати `payload` у JSON-рядок через `serde_json::to_string`.
  - Відкрити файл у режимі дописування (`OpenOptions::new().append(true)`) та додати рядок:
    `| {date} | {action_name} | {description} | \`{json}\` |`

### 3. Інтеграція викликів журналування (`model.rs`, `Board`)
- У функції `create_ticket`: викликати `append_log_entry` із `ActionPayload::CreateTicket`.
- У функції `move_ticket`: викликати з `ActionPayload::ChangeStatus`.
- У функції `update_ticket`: викликати з `ActionPayload::UpdateTicket` або `AssignTicket`.
- У функції `add_comment` (чи де вона обробляється): викликати з `ActionPayload::AddComment`.
- У функції копіювання файлів до `attachment/`: викликати з `ActionPayload::AttachFile`.

## Тести
1. **Unit-тест для `UserConfig`**: перевірити, що при завантаженні конфігу без `machine_id` він автоматично генерується і зберігається.
2. **Unit-тест для `append_log_entry`**: перевірити створення файлу логу, його заголовків та додавання першого запису у правильному форматі.
3. **Integration Tests**: Перевірити, що при створенні тікету `create_ticket` та переміщенні `move_ticket` файл журналу успішно доповнюється. Всі існуючі тести мають проходити й надалі (слід переконатись, що тести використовують тимчасові директорії).

## Ручна перевірка
1. Запустити програму.
2. Створити тікет "Тест 1".
3. Перевірити, що в папці `Kanban/logs/` з'явився файл `log_{user}_{machine_id}.md` з таблицею і JSON в кінці рядка: `{"action":"CreateTicket", "id":"...", "title":"Тест 1"}`.
4. Перемістити тікет, додати коментар і прикріпити файл через UI або CLI.
5. Відкрити файл `.md` у будь-якому Markdown-переглядачі та перевірити, що таблиця відображається коректно і зручно для читання.
