# Plan - Internationalization using Gettext

## Objective
Translate the Slint Kanban application into Ukrainian and set up a robust internationalization (i18n) workflow using Gettext (.po format). This includes marking strings, extracting them, and loading translations at runtime.

## Changes

### 1. Dependencies (`Cargo.toml`)
- Add `tr` crate for Rust translation macros.
- Enable `gettext` feature for the `slint` crate.
- Add `gettext-rs` (usually pulled by `tr`).

### 2. Code Changes (Rust)
- Wrap all user-facing strings in `src/main.rs`, `src/model.rs`, etc., with the `tr!` macro.
- Initialize translations in `main()` using `tr::tr_init!` and `slint::init_translations!`.
- Set up a standard directory structure for translation files: `i18n/uk/LC_MESSAGES/slint_kanban.mo`.

### 3. Code Changes (Slint)
- Update `ui/**/*.slint` to use the `@tr()` macro for all translatable labels, placeholders, and tooltips.

### 4. Extraction Script
- Create `scripts/extract-strings.sh` to automagically:
    1. Extract strings from `.rs` files using `xtr`.
    2. Extract strings from `.slint` files using `slint-tr-extractor`.
    3. Merge them into a single `po/slint_kanban.pot` template.

### 5. Build Process
- Add instructions or a script to compile `.po` to `.mo` using `msgfmt`.

## Research
- Document the findings in `research.md` regarding Gettext usage with Slint and Rust.

## Tests
- Verification that strings are extracted correctly.
- Manual verification of the Ukrainian translation in the GUI and CLI.
- Unit test to ensure translation initialization doesn't crash.

## Manual verification
- Set `LANG=uk_UA.UTF-8` and run the app to see translated UI.
- Run CLI commands and check output.
