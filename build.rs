fn main() {
    cxx_build::bridge("src/main.rs") // читает #[cxx::bridge] из src/main.rs и готовит генерацию прокладок Rust↔C++
        .file("src/greeter.cc") // добавляет C++-файл src/greeter.cc в компиляцию
        .include("include") // добавляет папку include/ в пути поиска заголовков (-Iinclude)
        .std("c++17") // задаёт стандарт C++17 для компиляции C++-части
        .compile("o_070_cxx_bridge"); // компилирует всё в статическую библиотеку libo_070_cxx_bridge.a и линкует её
/*
Полный поток выполнения:
1. cargo build
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

2. ./target/.../o_070_cxx
       │
       ├─► ffi::greet("Rust")
       │       └─► Rust → C++ shim → greet(rust::Str) → rust::String → Rust String
       │
       └─► ffi::add(20, 22)
               └─► Rust → C++ shim → add(int32_t, int32_t) → int32_t → i32
*/

/*
Что попадает в libo_070_cxx_bridge.a

libo_070_cxx_bridge.a
├── greeter.o                  ← ваш C++-код (src/greeter.cc)
│   ├── greet(rust::Str)       ← реализация C++-функции
│   └── add(int32_t, int32_t)  ← реализация C++-функции
│
└── cxxbridge shims .o         ← сгенерированные CXX прокладки
    ├── Rust→C++: обёртки, вызывающие greet/add из Rust
    └── C++→Rust: обёртки, вызывающие Rust-функции из C++
    
*/


    // вернутся в пересоздание если изменился файл src/main.rs
    println!("cargo:rerun-if-changed=src/main.rs");
    // вернутся в пересоздание если изменился файл src/greeter.cc
    println!("cargo:rerun-if-changed=src/greeter.cc");
    // вернутся в пересоздание если изменился файл include/greeter.h
    println!("cargo:rerun-if-changed=include/greeter.h");
}
