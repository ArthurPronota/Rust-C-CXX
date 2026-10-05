# Мост Rust ↔ C++ через CXX

Этот проект демонстрирует **обратное направление** по сравнению с `o_070_cbinding`: не C++ вызывает Rust, а **Rust вызывает C++**. Мост описан в `#[cxx::bridge]`, а CXX автоматически генерирует прокладки и компилирует обе стороны.

---

## 🗂️ Структура проекта

```text
o_070_cxx/
├── Cargo.toml           # зависимости: cxx (runtime) + cxx-build (build)
├── build.rs             # сборочный скрипт: вызывает cxx_build::bridge(...)
├── src/
│   ├── main.rs          # #[cxx::bridge] + main() — точка входа Rust
│   └── greeter.cc       # реализация C++-функций greet, add
└── include/
    └── greeter.h        # объявления C++-функций (для CXX и C++)
```

Ключевые файлы:

| Файл | Роль |
|------|------|
| `src/main.rs` | описывает **мост** (`#[cxx::bridge]`) и вызывает C++ из `main()` |
| `include/greeter.h` | **объявления** C++-функций с обёртками `rust::String`, `rust::Str` |
| `src/greeter.cc` | **реализация** этих функций |
| `build.rs` | парсит мост, генерирует прокладки, компилирует C++ |
| `Cargo.toml` | подключает `cxx` и `cxx-build` |

---

## 🔄 Направление вызова

```
Rust (main.rs)  ──►  CXX shim  ──►  C++ (greeter.cc)
   ffi::greet()                      greet(rust::Str)
   ffi::add()                        add(int32_t, int32_t)
```

Это **Rust → C++**. В `o_070_cbinding` было наоборот: C → Rust. Оба варианта поддерживаются, но описываются по-разному.

---

## 📝 `Cargo.toml`

```toml
[package]
name = "o_070_cxx"
version = "0.1.0"
edition = "2024"

[dependencies]
cxx = "1.0"                 # runtime: макрос #[cxx::bridge], типы rust::String и т.п.

[build-dependencies]
cxx-build = "1.0"           # build-time: доступен в build.rs
```

Два крейта — две роли:

| Крейт | Секция | Когда работает | Что даёт |
|-------|--------|----------------|----------|
| `cxx` | `[dependencies]` | компиляция + рантайм | макрос `#[cxx::bridge]`, типы `CxxString`, `UniquePtr`, … |
| `cxx-build` | `[build-dependencies]` | в `build.rs` | функцию `cxx_build::bridge(...)` |

Без `cxx` не соберётся `main.rs`. Без `cxx-build` не соберётся `build.rs`. Оба обязательны.

---

## 📝 `src/main.rs`

```rust
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("greeter.h");

        fn greet(name: &str) -> String;
        fn add(left: i32, right: i32) -> i32;
    }
}

fn main() {
    let x = 20;
    let y = 22;

    let message = ffi::greet("Rust");
    let sum = ffi::add(x, y);

    println!("{message}");         // Hello from C++, Rust!
    println!("{x} + {y} = {sum}"); // 20 + 22 = 42
}
```

### `#[cxx::bridge]`

Макрос-атрибут из крейта `cxx`, превращающий модуль в FFI-мост. На этапе компиляции:

1. **Парсит** содержимое модуля `ffi`.
2. **Генерирует**:
   - Rust-код для вызова C++-функций;
   - C++-код прокладок (shims);
   - заголовок для C++-стороны (в `OUT_DIR`).
3. **Проверяет** соответствие типов — если что-то не стыкуется, ошибка компиляции.

### `unsafe extern "C++"`

| Часть | Значение |
|-------|----------|
| `unsafe` | вызовы небезопасны (C++ не даёт гарантий памяти) |
| `extern "C++"` | соглашение о вызове C++ — CXX берёт совместимость на себя |
| `{ ... }` | список C++-функций и типов, доступных из Rust |

Противоположный блок — `extern "Rust"` (Rust-функции, вызываемые из C++). Здесь он не нужен: весь C++-код предоставляется Rust'у.

### `include!("greeter.h")`

Директива CXX: «добавь `#include "greeter.h"` в сгенерированный C++-код». Нужна, чтобы CXX знал объявления C++-функций.

### Объявления функций

```rust
fn greet(name: &str) -> String;
fn add(left: i32, right: i32) -> i32;
```

Это **объявления без тела** — реализация в `greeter.cc`. CXX мапит типы:

| Rust | C++ |
|------|-----|
| `&str` | `rust::Str` |
| `String` | `rust::String` |
| `i32` | `std::int32_t` |

Имена `greet`, `add` **должны точно совпадать** с C++-именами в `greeter.h` / `greeter.cc`. Если нужно другое имя в Rust — `#[cxx_name = "..."]`.

### `main()`

```rust
let message = ffi::greet("Rust");   // Rust → C++ → Rust
let sum = ffi::add(x, y);           // Rust → C++
```

Вызовы выглядят как **обычные Rust-функции** — никаких `unsafe`, `CString`, `CStr`. CXX генерирует всю FFI-рутину автоматически.

---

## 📝 `include/greeter.h`

```cpp
#pragma once
#include <cstdint>
#include "rust/cxx.h"

rust::String greet(rust::Str name);
std::int32_t add(std::int32_t left, std::int32_t right);
```

| Строка | Что делает |
|--------|-----------|
| `#pragma once` | защита от повторного включения |
| `#include <cstdint>` | типы `std::int32_t` |
| `#include "rust/cxx.h"` | обёртки `rust::String`, `rust::Str` (поставляется CXX) |
| `rust::String greet(rust::Str)` | C++-сигнатура, соответствующая `fn greet(&str) -> String` в мосте |
| `std::int32_t add(...)` | примитивы, мапятся 1:1 на `i32` |

`greeter.h` — **мост со стороны C++**: C++-компилятор должен видеть эти объявления, чтобы слинковаться с Rust через сгенерированные прокладки.

---

## 📝 `src/greeter.cc`

```cpp
#include "greeter.h"
#include <string>

rust::String greet(rust::Str name) {
    std::string message = "Hello from C++, ";
    message.append(name.data(), name.size());
    message.push_back('!');
    return rust::String(message);
}

std::int32_t add(std::int32_t left, std::int32_t right) {
    return left + right;
}
```

### `greet`

1. `std::string message = "Hello from C++, "` — C++-строка.
2. `message.append(name.data(), name.size())` — добавляет Rust-строку **по указателю и длине**.
   - `rust::Str` — не C-строка, а срез (ptr + len), как Rust `&str`.
   - **Нельзя** `message += name.data()` — Rust-строки не NUL-терминированы.
3. `message.push_back('!')` — добавляет `!`.
4. `return rust::String(message)` — копирует `std::string` в Rust-аллокатор (разные аллокаторы → копия).

### `add`

```cpp
return left + right;
```

Просто сложение. `std::int32_t` ↔ Rust `i32` — **ABI-совместимые примитивы**, без обёрток, без копий.

---

## 📝 `build.rs`

```rust
fn main() {
    cxx_build::bridge("src/main.rs")   // парсит #[cxx::bridge], готовит прокладки
        .file("src/greeter.cc")        // добавляет C++-файл в компиляцию
        .include("include")            // добавляет -Iinclude для C++-компилятора
        .std("c++17")                  // -std=c++17 (или /std:c++17 на MSVC)
        .compile("o_070_cxx_bridge");  // компилирует в libo_070_cxx_bridge.a и линкует

    println!("cargo:rerun-if-changed=src/main.rs");    // перезапуск при изменении моста
    println!("cargo:rerun-if-changed=src/greeter.cc"); // перезапуск при изменении C++
    println!("cargo:rerun-if-changed=include/greeter.h"); // перезапуск при изменении заголовка
}
```

### Что делает цепочка

| Метод | Действие |
|-------|----------|
| `cxx_build::bridge("src/main.rs")` | читает `#[cxx::bridge]` из `main.rs`, готовит генерацию shims |
| `.file("src/greeter.cc")` | добавляет C++-реализацию в компиляцию |
| `.include("include")` | эквивалент `-Iinclude` |
| `.std("c++17")` | эквивалент `-std=c++17` (портируемо на MSVC) |
| `.compile("o_070_cxx_bridge")` | компилирует всё в `libo_070_cxx_bridge.a` и автоматически линкует с Rust |

### `cargo:rerun-if-changed`

Контракт между `build.rs` и `cargo`: «перезапускай меня только если изменился один из этих файлов». Без директив cargo перезапускает `build.rs` при изменении **любого** файла проекта — медленно. С неполным списком — баги при пересборке.

---

## 📦 Что попадает в `libo_070_cxx_bridge.a`

```
libo_070_cxx_bridge.a
├── greeter.o                  ← ваш C++-код (src/greeter.cc)
│   ├── greet(rust::Str)
│   └── add(int32_t, int32_t)
│
└── cxxbridge shims .o         ← сгенерированные CXX прокладки
    ├── Rust→C++: обёртки, вызывающие greet/add из Rust
    └── C++→Rust: обёртки, вызывающие Rust-функции из C++
```

Внутри — **только C++-часть** проекта. Rust-код (`main.rs`) компилируется отдельно `cargo`, а затем всё линкуется вместе.

---

## 🔁 Полный поток сборки и выполнения

### Сборка

```
cargo build
    │
    ├─► build.rs
    │       └─► cxx_build::bridge("src/main.rs")
    │               ├─► парсит #[cxx::bridge]
    │               ├─► генерирует Rust shims + C++ shims
    │               ├─► компилирует greeter.cc + shims → libo_070_cxx_bridge.a
    │               └─► говорит cargo: "линкуй эту .a"
    │
    └─► компилирует src/main.rs
            │
            └─► ffi::greet / ffi::add → вызовы в shims
```

### Выполнение

```
./target/.../o_070_cxx
    │
    ├─► ffi::greet("Rust")
    │       └─► Rust → C++ shim → greet(rust::Str) → rust::String → Rust String
    │
    └─► ffi::add(20, 22)
            └─► Rust → C++ shim → add(int32_t, int32_t) → int32_t → i32
```

**Вывод программы:**

```
Hello from C++, Rust!
20 + 22 = 42
```

---

## 🧩 Ключевые особенности CXX в этом проекте

### 1. Rust вызывает C++ — это возможно

CXX поддерживает оба направления. Здесь — `unsafe extern "C++"`, Rust зовёт C++.

### 2. Автоматический маппинг типов

| Rust | C++ | Как передаётся |
|------|-----|----------------|
| `&str` | `rust::Str` | ptr + len, без копии |
| `String` | `rust::String` | владеющая, копия при конвертации |
| `i32` | `std::int32_t` | напрямую, ABI-совместимо |

### 3. Никакого `unsafe` в пользовательском коде

```rust
let message = ffi::greet("Rust");   // ← безопасный вызов
```

Хотя внутри сгенерированных shims есть `unsafe`, снаружи всё выглядит как обычный Rust.

### 4. Один `.a` — C++-половина моста

`libo_070_cxx_bridge.a` содержит C++-код + shims. Rust компилируется отдельно и линкуется с этим `.a` автоматически (cargo знает про `.a` благодаря директивам от `cxx-build`).

### 5. Единый источник истины — мост в `main.rs`

И сигнатуры C++ (`greeter.h`), и реализации (`greeter.cc`), и вызовы в Rust (`ffi::greet`) **согласованы** через `#[cxx::bridge]`. CXX проверяет, что типы совпадают, и генерирует связующий код.

---

## ⚖️ Сравнение с `o_070_cbinding`

| | `o_070_cbinding` (cbindgen) | `o_070_cxx` (CXX) |
|---|---|---|
| Направление | C → Rust | Rust → C++ |
| Где описан интерфейс | атрибуты Rust (`#[repr(C)]`, `#[no_mangle]`) | `#[cxx::bridge]` |
| Что генерируется | `.h` (только заголовок) | Rust shims + C++ shims + `.a` |
| Кто следит за типами | **вы** (вручную) | **CXX** (проверяет) |
| Нужен `unsafe` в коде | да | нет (в пользовательском коде) |
| Строки | `*const c_char` + `CStr` | `rust::Str` / `rust::String` |
| Зависимости | `cbindgen` в `[build-dependencies]` | `cxx` + `cxx-build` |
| Сборка | `cargo build` + `cbindgen` CLI | `cargo build` (всё авто) |

---

## ✅ Итог

Проект `o_070_cxx` — пример **современного безопасного FFI** между Rust и C++ через CXX:

1. **`main.rs`** описывает мост (`#[cxx::bridge]`) и вызывает C++-функции как обычные Rust-функции.
2. **`greeter.h`** даёт C++-объявления с обёртками CXX (`rust::Str`, `rust::String`).
3. **`greeter.cc`** реализует C++-функции в идиоматичном C++.
4. **`build.rs`** автоматически генерирует прокладки и компилирует C++ в `libo_070_cxx_bridge.a`.
5. **`Cargo.toml`** подключает runtime (`cxx`) и build-time (`cxx-build`) части.
6. Всё линкуется в один бинарь, где Rust и C++ вызывают друг друга **без ручного `unsafe`**.

**Ключевая ценность:** CXX делает FFI **безопасным и идиоматичным** с обеих сторон. Rust-программист не пишет `unsafe` и не думает о C-строках, C++-программист не думает о Rust-рантайме. Весь glue-код генерируется автоматически из единого описания моста.
