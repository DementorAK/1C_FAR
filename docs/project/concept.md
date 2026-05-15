# **Архитектура и стратегии разработки мультиплатформенных плагинов для FAR Manager в экосистеме Windows и Linux**

Эволюция ортогональных файловых менеджеров (OFM), к которым относится FAR Manager, прошла путь от узкоспециализированных инструментов для Windows до кроссплатформенных оболочек. В центре этой трансформации стоит задача обеспечения расширяемости через систему плагинов. Перенос этой инфраструктуры на Unix-подобные системы через проекты far2l и far2m создал условия для поддержки единой кодовой базы, где на смену традиционному C++ приходят более безопасные и современные языки, такие как Rust.1

## **Исторические предпосылки и технологическая база портов FAR Manager на Linux**

FAR Manager изначально был ориентирован исключительно на платформу Win32. После открытия исходных кодов в 2007 году проект far2l стал первой успешной попыткой создания полноценного порта для Linux, macOS и BSD.4

Разработчик far2l выбрал в качестве основы исходный код FAR Manager версии 2.0, так как его кодовая база была компактнее и проще для реализации слоя абстракции над системными вызовами Linux.6 Однако потребность в интеграции системы макросов и плагинов на языке Lua (стандарт FAR 3\) привела к появлению форка far2m, который развивается параллельно и ориентирован на расширенный API и глубокую автоматизацию.2

### **Сравнительный анализ версий FAR Manager и их портов**

| Характеристика | FAR Manager 3 (Windows) | FAR Manager 2 (Windows) | far2l (Linux/Unix) | far2m (Linux/Unix) |
| :---- | :---- | :---- | :---- | :---- |
| **Базовая версия** | FAR 3.0 | FAR 2.0 | FAR 2.0 | far2l (форк) |
| **Ядро макросов** | LuaJIT | Нет | Собственное (v2) | LuaJIT (от FAR 3\) |
| **Лицензия** | BSD-3-Clause | BSD-3-Clause | GNU GPLv2 | GNU GPLv2 |
| **API плагинов** | FAR 3 Plugin API (GUID) | FAR 2 Plugin API (строки) | FAR 2 Plugin API (расширенный) | FAR 2 Plugin API (расширенный+) |
| **Совместимость API** | ❌ Несовместимо с FAR 2 | ❌ Несовместимо с FAR 3 | ✅ Совместимо с FAR 2 | ✅ Совместимо с FAR 2/far2l |

## **Архитектурная парадигма: Dual-API плагин**

Ключевое архитектурное решение проекта — поддержка **двух несовместимых версий Plugin API** через единую кодовую базу:

### **FAR 3 Plugin API (Windows)**

FAR 3 ввёл архитектурный разрыв (API break) по отношению к FAR 2:
- **Идентификация через GUID** — каждый плагин, диалог, пункт меню идентифицируется 128-битным UUID
- **Упакованные параметры** — экспортируемые функции получают указатели на структуры вместо отдельных параметров
- **Новые функции** — `GetGlobalInfoW`, `OpenW`, `AnalyseW` (вместо `OpenPluginW`, `OpenFilePluginW`)
- **SQLite-настройки** — хранение через SettingsControl API
- **Расширенные цвета** — поддержка TrueColor RGBA

### **FAR 2 Plugin API (far2l, far2m, FAR 2 Windows)**

FAR 2 API используется в Linux-портах (far2l, far2m):
- **Строковая идентификация** — плагины идентифицируются строковыми именами и числовым SysID
- **Прямые параметры** — функции получают параметры напрямую (`OpenPluginW(OpenFrom, Item)`)
- **OpenFilePluginW** — анализ файлов совмещён с открытием
- **Реестр/INI-настройки** — без SQLite
- **WinPort** — слой совместимости Win32 API на Linux
- **`wchar_t` = 4 байта** — UTF-32 вместо UTF-16

### **Принципы организации дистрибутива плагина**

Плагин поставляется в виде единого каталога:

```
far1c/
├── far1c.dll           -- Windows x64 (FAR Manager 3)
├── far1c.so            -- Linux x64 (far2l / far2m)
├── far1c_en.lng        -- English localization
├── far1c_ru.lng        -- Russian localization
└── README.md
```

Хост-приложение при сканировании каталога автоматически выбирает совместимый бинарный файл (Windows проигнорирует .so, а Linux — .dll).3

## **Выбор языка: Rust как современная альтернатива C++**

Традиционно плагины писались на C++, что требовало ручного управления памятью и сложной настройки кросс-компиляции. Rust предлагает те же возможности по производительности и нативности, но с гораздо более строгими гарантиями безопасности и удобным инструментарием.13

### **Преимущества Rust в разработке плагинов**

1. **Безопасность памяти:** Rust предотвращает целые классы ошибок (нулевые указатели, выход за границы массива, утечки памяти), которые часто встречаются в C++ плагинах.  
2. **FFI и совместимость:** Rust обладает отличной поддержкой интерфейса внешних функций (FFI). Используя атрибут \#\[no\_mangle\] и extern "C", функции Rust становятся полностью совместимыми с C-интерфейсом FAR Manager.  
3. **Условная компиляция:** Механизм `#[cfg(feature = "...")]` позволяет элегантно выбирать нужную версию API при компиляции.
4. **Единая система сборки:** Cargo одинаково работает на Windows и Linux, упрощая управление зависимостями и сборку бинарных файлов (.dll и .so) из одного исходного кода.

### **Сравнение подходов к разработке**

| Параметр | Нативный C++ | Rust (Рекомендуется) | Скриптовый Lua |
| :---- | :---- | :---- | :---- |
| **Безопасность** | Низкая (ручное управление) | Высокая (Memory Safe) | Средняя (Sandbox) |
| **Скорость** | Максимальная | Максимальная | Ниже (интерпретатор) |
| **Dual-API поддержка** | Сложная (препроцессор) | Элегантная (features + traits) | Минимальная (скрипты) |
| **Интеграция с ОС** | Прямая | Прямая | Через API |

## **Практическая реализация на Rust**

Для создания мультиплатформенного плагина на Rust применяется стратегия **Feature-gated Dual API**:

* **Изоляция платформозависимого кода:** Cargo features (`far3`, `far2`) определяют, какая версия API-биндингов и экспортируемых функций включается в сборку.
* **Общий трейт FarHost:** Абстракция над различиями API (строковые конвертации, диалоги, панели) через Rust traits.
* **Структура проекта:** Бизнес-логика (парсинг/запись контейнеров 1С) полностью изолирована от API-слоя и не зависит от версии FAR.

```bash
# Сборка для Windows (FAR Manager 3) — по умолчанию
cargo build --release

# Сборка для Linux (far2l / far2m)
cargo build --release --features far2 --no-default-features
```

## **Проблемы и ограничения при переносе**

Разработчик должен учитывать:

1. **Размерность wchar_t** — 2 байта (UTF-16) на Windows, 4 байта (UTF-32) на Linux. Требуются отдельные конвертеры строк.
2. **Различия в calling conventions** — `extern "system"` для Windows, `extern "C"` для Linux.
3. **Разные структуры данных** — PluginPanelItem, PluginInfo и другие структуры имеют разную раскладку полей.
4. **WinPort** — плагины для far2l линкуются с символами из исполняемого файла far2l, а не со статической библиотекой.
5. **Пути и кодировки** — разделители \\ vs /, нормализация путей.

## **Выводы и рекомендации**

1. **Rust — оптимальный выбор:** Для разработки бинарных плагинов Rust является наиболее надежным и современным вариантом, сочетающим производительность C++ и безопасность современных языков.  
2. **Dual-API через Cargo features:** Единая кодовая база с feature-gated компиляцией — наиболее элегантный способ поддержки FAR 3 и far2l/far2m.
3. **Формат поставки:** Используйте единый каталог, содержащий MyPlugin.dll (Windows x64) и MyPlugin.so (Linux far2l/far2m).3  
4. **Целевые порты:** FAR Manager 3 (Windows) — primary, far2l (Linux) — primary, far2m — совместимость с far2l.

Переход на Rust в экосистеме FAR Manager позволяет создавать стабильные инструменты, минимизируя время на отладку низкоуровневых ошибок и упрощая поддержку кода для разных операционных систем.

#### **Источники**

1. far2l.git/summary \- Public Git Hosting, дата последнего обращения: апреля 5, 2026, [https://www.repo.or.cz/far2l.git](https://www.repo.or.cz/far2l.git)  
2. shmuz/far2m: Linux port of FAR2 with FAR3 macro system and extended plugins' API \- GitHub, дата последнего обращения: апреля 5, 2026, [https://github.com/shmuz/far2m](https://github.com/shmuz/far2m)  
3. How to make a FAR plug-in using Visual C++ \- Far Manager Documentation, дата последнего обращения: апреля 5, 2026, [https://documentation.help/Far-Manager/far\_plugin.html](https://documentation.help/Far-Manager/far_plugin.html)  
4. Far Manager download | SourceForge.net, дата последнего обращения: апреля 5, 2026, [https://sourceforge.net/projects/farmanager/](https://sourceforge.net/projects/farmanager/)  
5. Enjoy two-panel file management on Linux with far2l | Opensource.com, дата последнего обращения: апреля 5, 2026, [https://opensource.com/article/22/12/linux-file-manager-far2l](https://opensource.com/article/22/12/linux-file-manager-far2l)  
6. Why not base off Far 3 instead of Far 2? · elfmz far2l · Discussion \#1890 \- GitHub, дата последнего обращения: апреля 5, 2026, [https://github.com/elfmz/far2l/discussions/1890](https://github.com/elfmz/far2l/discussions/1890)  
7. Far Manager for Linux and MacOS | Roman's blog, дата последнего обращения: апреля 5, 2026, [https://blog.kirillov.cc/posts/far-manager/](https://blog.kirillov.cc/posts/far-manager/)  
8. elfmz/far2l: Linux port of FAR v2 \- GitHub, дата последнего обращения: мая 14, 2026, [https://github.com/elfmz/far2l](https://github.com/elfmz/far2l)  
9. Porting is a Delicate Matter: Checking Far Manager under Linux \- PVS-Studio, дата последнего обращения: мая 14, 2026, [https://pvs-studio.com/en/blog/posts/cpp/0478/](https://pvs-studio.com/en/blog/posts/cpp/0478/)  
10. FarManager/far/plugin.hpp at master \- GitHub, дата последнего обращения: мая 14, 2026, [https://github.com/FarGroup/FarManager/blob/master/far/plugin.hpp](https://github.com/FarGroup/FarManager/blob/master/far/plugin.hpp)
11. far2l/far2l/far2sdk/farplug-wide.h at master · elfmz/far2l · GitHub, дата последнего обращения: мая 15, 2026, [https://github.com/elfmz/far2l/blob/master/far2l/far2sdk/farplug-wide.h](https://github.com/elfmz/far2l/blob/master/far2l/far2sdk/farplug-wide.h)
12. dpelevin/farmanager-api-rust-bindings: Far Manager Plugins API bindings for Rust \- GitHub, дата последнего обращения: мая 14, 2026, [https://github.com/dpelevin/farmanager-api-rust-bindings](https://github.com/dpelevin/farmanager-api-rust-bindings)
13. Plugin System: Building a Cross-Platform C++ Solution \- Pav Creations, дата последнего обращения: апреля 5, 2026, [https://pavcreations.com/plugin-system-building-a-cross-platform-c-solution/](https://pavcreations.com/plugin-system-building-a-cross-platform-c-solution/)