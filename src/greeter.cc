/*
Это реализация C++-функций greet и add, объявленных в greeter.h. Их вызывает Rust через 
мост CXX. C++-код пишется как обычный C++ — про Rust он «не знает», только использует 
обёртки rust::String и rust::Str.
*/

// Подключает свой заголовок с объявлениями
#include "greeter.h"

// Стандартный C++-заголовок для std::string — владеющей строки C++.
#include <string>

/*
Часть	        Что означает
--------------- -----------------------------------------------------------
rust::String	возвращаемый тип — Rust-строка (владеющая)
greet	        имя функции — должно совпадать с именем в bridge-модуле
rust::Str       name	параметр — заимствованная Rust-строка (&str в Rust)

*/
rust::String greet(rust::Str name) {
    std::string message = "Hello from C++, ";
    message.append(name.data(), name.size());
    message.push_back('!');
    return rust::String(message);
}

/*
Часть	        Что означает
--------------- -----------------------------------------
std::int32_t	32-битное знаковое целое, аналог Rust i32
add	            имя функции — совпадает с bridge-модулем
left, right	    параметры
*/
std::int32_t add(std::int32_t left, std::int32_t right) {
    return left + right;
}
