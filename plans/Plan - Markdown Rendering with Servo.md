# План реалізації - Відображення Markdown через Servo WebView (Опціонально)

Цей план описує інтеграцію двигуна Servo для рендерингу Markdown як опціональної можливості, що активується через Cargo feature.

## Мета
Додати можливість відображення форматованого Markdown через `WebView` (Servo), зберігши при цьому можливість збірки проекту без Servo (з використанням стандартного `Text`) для систем з обмеженими ресурсами або без необхідних графічних драйверів.

## Зміни

### 1. Залежності та Features (`Cargo.toml`)
- Додати feature `servo`:
  ```toml
  [features]
  default = []
  servo = ["dep:libservo", "dep:wgpu", "dep:pulldown-cmark", "dep:gl", "dep:glow", "dep:url", "dep:smol", "dep:spin_on", "dep:winit"]

  [dependencies]
  libservo = { git = "https://github.com/servo/servo", rev = "...", optional = true }
  wgpu = { version = "28.0", optional = true }
  pulldown-cmark = { version = "0.10", optional = true }
  # ... інші залежності Servo як optional = true
  ```

### 2. Структура проекту та Умовна компіляція
- Модуль `src/webview/` буде включений лише за умови `#[cfg(feature = "servo")]`.
- Усі специфічні для Servo виклики в `main.rs`, `controller.rs` та `lib.rs` будуть обгорнуті в `#[cfg(feature = "servo")]`.
- Створити заглушку або альтернативну реалізацію для `WebView`, якщо feature вимкнена.

### 3. Ініціалізація (`src/main.rs`)
- `run_gui` буде мати дві гілки:
    - **З `servo`**: Ручне налаштування WGPU та ініціалізація WebView.
    - **Без `servo`**: Стандартний запуск Slint (використовуючи `slint::run_window`).

### 4. UI (`ui/dialogs/ticket_view.slint` та `ui/common.slint`)
- Використовувати умовний імпорт та відображення компонентів (через властивості або вкладені елементи, які Slint ігнорує, якщо вони не використовуються).
- Оскільки Slint не підтримує `#[cfg]` безпосередньо в `.slint` файлах, ми будемо керувати видимістю через властивість `has-webview`, яку встановлює Rust код.
- Якщо `has-webview` == false, буде відображатись звичайний `Text`.

### 5. Контролер та Тікети
- Якщо `servo` увімкнено: `Markdown -> HTML -> WebView`.
- Якщо `servo` вимкнено: `Markdown -> Plain Text (або простий Slint RichText) -> Text`.

## Ризики та складнощі
- **Подвійна підтримка**: Необхідно підтримувати два методи відображення контенту.
- **WGPU**: Навіть якщо Servo вимкнено, ми маємо переконатись, що стандартний бекенд Slint працює коректно.

## Тестування
1. **Без функцій**: `cargo test` — перевірка, що все працює як раніше.
2. **З feature**: `cargo test --features servo` — перевірка нової логіки.

## Ручна перевірка
1. `cargo run` — перевірка стандартного текстового вигляду.
2. `cargo run --features servo` — перевірка роботи WebView.
