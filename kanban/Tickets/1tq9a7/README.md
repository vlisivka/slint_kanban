---
title: Помилка в розборі тексту тікету
created_at: 2026-06-21 14:33:40
updated_at: 2026-06-21 14:33:40
assigned_to: "user"
author: "user"
points: 0
attachment_count: 0
---
```
thread 'main' (1879801) panicked at src/model/ticket.rs:66:53:
end byte index 588 is not a char boundary; it is inside 'е' (bytes 587..589) of ````
Project root: /home/vlisivka/workspace/po-tools-rust
Використовується корінь Канбан: /home/vlisivka/workspace/po-tools-rust/kanban
Warning: Failed to load ticket at "/home/vlisivka/workspace/po-tools-rust/kanban/Tickets/llnz`[...]
stack backtrace:
   0:     0x560fd678be5a - <<std[e28293b1aa0f68bd]::sys::backtrace::BacktraceLock>::print::DisplayBacktrace as core[c1f1a4ba060b9bfa]::fmt::Display>::fmt
   1:     0x560fd67a711a - core[c1f1a4ba060b9bfa]::fmt::write
   2:     0x560fd6793e62 - <std[e28293b1aa0f68bd]::sys::stdio::unix::Stderr as std[e28293b1aa0f68bd]::io::Write>::write_fmt
   3:     0x560fd67695df - std[e28293b1aa0f68bd]::panicking::default_hook::{closure#0}
   4:     0x560fd6783d51 - std[e28293b1aa0f68bd]::panicking::default_hook
   5:     0x560fd6783fcb - std[e28293b1aa0f68bd]::panicking::panic_with_hook
   6:     0x560fd6769698 - std[e28293b1aa0f68bd]::panicking::panic_handler::{closure#0}
   7:     0x560fd67606d9 - std[e28293b1aa0f68bd]::sys::backtrace::__rust_end_short_backtrace::<std[e28293b1aa0f68bd]::panicking::panic_handler::{closure#0}, !>
   8:     0x560fd676a54d - __rustc[b7974e8690430dd9]::rust_begin_unwind
   9:     0x560fd67a7a2c - core[c1f1a4ba060b9bfa]::panicking::panic_fmt
  10:     0x560fd67a7546 - core[c1f1a4ba060b9bfa]::str::slice_error_fail_rt
  11:     0x560fd67a71ba - core[c1f1a4ba060b9bfa]::str::slice_error_fail
  12:     0x560fd570c08b - slint_kanban::model::ticket::Ticket::extract_references::h931b2afb15832c4b
  13:     0x560fd5b26e8d - slint_kanban::into_slint_ticket::h7de989734315c875
  14:     0x560fd5b23083 - slint_kanban::controller::AppController::handle_request_full_ticket::h49a713616222e29f
  15:     0x560fd56daf62 - i_slint_core::callbacks::Callback<Arg,Ret>::set_handler::{{closure}}::h3997d0dc9ad66256
  16:     0x560fd584e766 - i_slint_core::callbacks::Callback<Arg,Ret>::call::h0fb4d64c204dd7dc
  17:     0x560fd5a7a5c0 - core::ops::function::FnOnce::call_once::h983510f37209ae5c
  18:     0x560fd584f035 - i_slint_core::callbacks::Callback<Arg,Ret>::call::hbc5b69632454cab6
  19:     0x560fd5ad3236 - core::ops::function::FnOnce::call_once::hed5688808ba54ebd
  20:     0x560fd584f035 - i_slint_core::callbacks::Callback<Arg,Ret>::call::hbc5b69632454cab6
  21:     0x560fd59e6d93 - core::ops::function::FnOnce::call_once::h108f6288fda91308
  22:     0x560fd637a205 - i_slint_core::callbacks::Callback<Arg,Ret>::call::h6b15870090a575e6
  23:     0x560fd63483f1 - <i_slint_core::items::input_items::TouchArea as i_slint_core::items::Item_vtable_mod::Item>::input_event::h77a0eb50f81b0f33
  24:     0x560fd63516e3 - i_slint_core::window::WindowInner::process_mouse_input::hca8f0e8cf33636aa
  25:     0x560fd5cfde1b - <i_slint_backend_winit::event_loop::EventLoopState as winit::application::ApplicationHandler<i_slint_backend_winit::SlintEvent>>::window_event::hd0427a5c60f812e6
  26:     0x560fd5ce22cd - core::ops::function::impls::<impl core::ops::function::FnMut<A> for &mut F>::call_mut::he1c2743356a92a88
  27:     0x560fd5cff932 - core::ops::function::impls::<impl core::ops::function::FnMut<A> for &mut F>::call_mut::h52a60ac8d1d91dee
  28:     0x560fd5d70eb0 - winit::platform_impl::linux::x11::event_processor::EventProcessor::xinput2_button_input::h68765bc4a4f01118
  29:     0x560fd5d6196e - winit::platform_impl::linux::x11::event_processor::EventProcessor::process_xevent::hb28785ccbeded26b
  30:     0x560fd5d5d334 - winit::platform_impl::linux::x11::event_processor::EventProcessor::process_event::hb94209d068b39678
  31:     0x560fd5d0755f - winit::platform_impl::linux::x11::EventLoop<T>::single_iteration::h73dcfffa800d0c2a
  32:     0x560fd5d08342 - winit::platform_impl::linux::x11::EventLoop<T>::poll_events_with_timeout::h82802f7a2c72b3c3
  33:     0x560fd5d05456 - winit::platform_impl::linux::x11::EventLoop<T>::pump_events::hcdc3382123ed154e
  34:     0x560fd5d055de - winit::platform_impl::linux::x11::EventLoop<T>::run_on_demand::h56837853d6b536ee
  35:     0x560fd5dc72e3 - winit::platform::run_on_demand::EventLoopExtRunOnDemand::run_app_on_demand::h99dec71ccd362b64
  36:     0x560fd5cfeffc - i_slint_backend_winit::event_loop::EventLoopState::run::h1fb19ff42c908af2
  37:     0x560fd5dda023 - <i_slint_backend_winit::Backend as i_slint_core::platform::Platform>::run_event_loop::h66cc5565a6cdad7e
  38:     0x560fd5cd9c24 - std::thread::local::LocalKey<T>::with::h5a35860f9d9a6bb5
  39:     0x560fd5cd9f41 - slint::run_event_loop::hca7465c65d2cebaa
  40:     0x560fd5aeb4f2 - <slint_kanban::slint_generatedApp::App as i_slint_core::api::ComponentHandle>::run::h4121ae8594015c2d
  41:     0x560fd56ebcad - slint_kanban::run_gui::h232cb47c83a1d2ee
  42:     0x560fd56eb37e - slint_kanban::main::h36772f9021c58052
  43:     0x560fd56e3853 - std::sys::backtrace::__rust_begin_short_backtrace::hf7ee7d5518c17978
  44:     0x560fd56ce66d - std::rt::lang_start::{{closure}}::h1f1a258e2089c523
  45:     0x560fd6782c14 - std[e28293b1aa0f68bd]::rt::lang_start_internal
  46:     0x560fd56eceb5 - main
  47:     0x7f2d6e63b5b5 - __libc_start_call_main
  48:     0x7f2d6e63b668 - __libc_start_main_alias_1
  49:     0x560fd56cc905 - _start
  50:                0x0 - <unknown>
```

## Resolution

### Root Cause
Методи `extract_references()` у `Ticket` та `Comment` використовували байтове зрізання рядка:
`&self.description[actual_pos + 1..actual_pos + 7]`. Коли індекс `actual_pos + 7` потрапляв всередину багатобайтового UTF-8 символу (наприклад, кириличної літери), це викликало panic.

### Fix
Замінено байтове зрізання на ітерацію через `char_indices()`:
- Знаходимо `#` через `char_indices().find()` — гарантовано на межі символу.
- Збираємо наступні 6 символів через `.chars().take(6).collect()` — уникнуто байтового зрізання.
- Перевіряємо, що зібрано рівно 6 символів і всі вони ASCII-малі літери/цифри.

### Files Changed
- `src/model/ticket.rs` — `extract_references()` (lines 60-78)
- `src/model/comment.rs` — `extract_references()` (lines 24-43)
- `src/model/tests/ticket_tests.rs` — додано тест `test_extract_references_no_panic_on_unicode_byte_boundary`

### Tests
Всі 58 тестів пройдено, включаючи новий тест, що імітує сценарій з багатобайтовим символом після `#`.