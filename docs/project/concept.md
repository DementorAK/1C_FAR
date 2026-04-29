# **Архитектура и стратегии разработки мультиплатформенных плагинов для FAR Manager в экосистеме Windows и Linux**

Эволюция ортогональных файловых менеджеров (OFM), к которым относится FAR Manager, прошла путь от узкоспециализированных инструментов для Windows до кроссплатформенных оболочек. В центре этой трансформации стоит задача обеспечения расширяемости через систему плагинов. Перенос этой инфраструктуры на Unix-подобные системы через проекты far2l и far2m создал условия для поддержки единой кодовой базы, где на смену традиционному C++ приходят более безопасные и современные языки, такие как Rust.1

## **Исторические предпосылки и технологическая база портов FAR Manager на Linux**

FAR Manager изначально был ориентирован исключительно на платформу Win32. После открытия исходных кодов в 2007 году проект far2l стал первой успешной попыткой создания полноценного порта для Linux, macOS и BSD.4

Разработчик far2l выбрал в качестве основы исходный код FAR Manager версии 2.0, так как его кодовая база была компактнее и проще для реализации слоя абстракции над системными вызовами Linux.6 Однако потребность в интеграции системы макросов и плагинов на языке Lua (стандарт FAR 3\) привела к появлению форка far2m, который развивается параллельно и ориентирован на расширенный API и глубокую автоматизацию.2

### **Сравнительный анализ версий FAR Manager и их портов**

| Характеристика | FAR Manager (Windows) | far2l (Linux/Unix) | far2m (Linux/Unix) |
| :---- | :---- | :---- | :---- |
| **Базовая версия** | FAR 2.0 / 3.0 | FAR 2.0 | far2l (форк) |
| **Ядро макросов** | LuaJIT (в FAR 3\) | Собственное (v2) | LuaJIT (от FAR 3\) |
| **Лицензия** | BSD-3-Clause | GNU GPLv2 | GNU GPLv2 |
| **API плагинов** | Стандартный Windows API | MultiAPI (слой совместимости) | Расширенный MultiAPI |

## **Архитектура мультиплатформенного плагина: Единство каталога и различие бинарных файлов**

Технически возможно поставлять плагин в виде единого каталога, который будет работать в разных ОС.3 Хост-приложение при сканировании каталога автоматически выбирает совместимый бинарный файл, игнорируя неподходящие форматы (например, Windows проигнорирует .so, а Linux — .dll).3

### **Принципы организации дистрибутива плагина**

Для успешной загрузки плагин должен экспортировать стандартные функции (такие как SetStartupInfo, GetPluginInfo, OpenPlugin) через интерфейс C (extern "C").3 В портах far2l и far2m эти функции сохраняют свои сигнатуры, используя слой совместимости для маскировки типов данных Windows под Unix-аналоги.1

## **Выбор языка: Rust как современная альтернатива C++**

Традиционно плагины писались на C++, что требовало ручного управления памятью и сложной настройки кросс-компиляции. Rust предлагает те же возможности по производительности и нативности, но с гораздо более строгими гарантиями безопасности и удобным инструментарием.13

### **Преимущества Rust в разработке плагинов**

1. **Безопасность памяти:** Rust предотвращает целые классы ошибок (нулевые указатели, выход за границы массива, утечки памяти), которые часто встречаются в C++ плагинах.  
2. **FFI и совместимость:** Rust обладает отличной поддержкой интерфейса внешних функций (FFI). Используя атрибут \#\[no\_mangle\] и extern "C", функции Rust становятся полностью совместимыми с C-интерфейсом FAR Manager.  
3. **Готовые привязки (Bindings):** Существуют открытые проекты, такие как farmanager-api-rust-bindings, которые предоставляют готовые обертки над API FAR Manager. Они поддерживают макросы для упрощения разработки, например \#\[derive(FarPlugin)\] для автоматической генерации структур.  
4. **Единая система сборки:** Вместо настройки сложных Makefile или CMake, Rust использует Cargo, который одинаково работает на Windows и Linux, упрощая управление зависимостями и сборку бинарных файлов (.dll и .so) из одного исходного кода.

### **Сравнение подходов к разработке**

| Параметр | Нативный C++ | Rust (Рекомендуется) | Скриптовый Lua |
| :---- | :---- | :---- | :---- |
| **Безопасность** | Низкая (ручное управление) | Высокая (Memory Safe) | Средняя (Sandbox) |
| **Скорость** | Максимальная | Максимальная | Ниже (интерпретатор) |
| **Сложность портирования** | Высокая (нужен CMake) | Низкая (Cargo \+ FFI) | Минимальная (скрипты) |
| **Интеграция с ОС** | Прямая | Прямая | Через API |

## **Практическая реализация на Rust**

Для создания мультиплатформенного плагина на Rust рекомендуется следующая стратегия:

* **Изоляция платформозависимого кода:** Используйте условную компиляцию Rust (\#\[cfg(windows)\] и \#\[cfg(target\_os \= "linux")\]) для обработки различий в системных вызовах (например, работа с путями или реестром).  
* **Использование MultiAPI:** При сборке под Linux-порты ориентируйтесь на заголовки multiapi, которые обеспечивают совместимость с классическим API FAR.5  
* **Структура проекта:** Cargo позволяет собирать библиотеку (cdylib), которая на выходе даст нужный формат файла в зависимости от целевой платформы.

## **Проблемы и ограничения при переносе**

Разработчик должен учитывать различия в работе с путями (разделители \\ против /) и кодировками.16 Хотя FAR традиционно использует UTF-16 (wchar\_t), Rust нативно работает с UTF-8, что требует корректного преобразования строк при взаимодействии с API через соответствующие типам данные в FFI-обертках.2

## **Выводы и рекомендации**

1. **Rust — оптимальный выбор:** Для разработки бинарных плагинов Rust является наиболее надежным и современным вариантом, сочетающим производительность C++ и безопасность современных языков.  
2. **Формат поставки:** Используйте единый каталог, содержащий MyPlugin.dll (Windows x86/x64) и MyPlugin.so (Linux far2l/far2m).3  
3. **Целевые порты:** Основным ориентиром для Linux остается far2l, однако для пользователей, активно использующих Lua-макросы, поддержка far2m будет существенным преимуществом.2

Переход на Rust в экосистеме FAR Manager позволяет создавать стабильные инструменты, минимизируя время на отладку низкоуровневых ошибок и упрощая поддержку кода для разных операционных систем.

#### **Источники**

1. far2l.git/summary \- Public Git Hosting, дата последнего обращения: апреля 5, 2026, [https://www.repo.or.cz/far2l.git](https://www.repo.or.cz/far2l.git)  
2. shmuz/far2m: Linux port of FAR2 with FAR3 macro system and extended plugins' API \- GitHub, дата последнего обращения: апреля 5, 2026, [https://github.com/shmuz/far2m](https://github.com/shmuz/far2m)  
3. How to make a FAR plug-in using Visual C++ \- Far Manager Documentation, дата последнего обращения: апреля 5, 2026, [https://documentation.help/Far-Manager/far\_plugin.html](https://documentation.help/Far-Manager/far_plugin.html)  
4. Far Manager download | SourceForge.net, дата последнего обращения: апреля 5, 2026, [https://sourceforge.net/projects/farmanager/](https://sourceforge.net/projects/farmanager/)  
5. Enjoy two-panel file management on Linux with far2l | Opensource.com, дата последнего обращения: апреля 5, 2026, [https://opensource.com/article/22/12/linux-file-manager-far2l](https://opensource.com/article/22/12/linux-file-manager-far2l)  
6. Why not base off Far 3 instead of Far 2? · elfmz far2l · Discussion \#1890 \- GitHub, дата последнего обращения: апреля 5, 2026, [https://github.com/elfmz/far2l/discussions/1890](https://github.com/elfmz/far2l/discussions/1890)  
7. Far Manager for Linux and MacOS | Roman's blog, дата последнего обращения: апреля 5, 2026, [https://blog.kirillov.cc/posts/far-manager/](https://blog.kirillov.cc/posts/far-manager/)  
8. Far Manager: files and archives in Windows \- Hacker News, дата последнего обращения: апреля 5, 2026, [https://news.ycombinator.com/item?id=37101026](https://news.ycombinator.com/item?id=37101026)  
9. How to install a plugin in far manager \- Super User, дата последнего обращения: апреля 5, 2026, [https://superuser.com/questions/1616957/how-to-install-a-plugin-in-far-manager](https://superuser.com/questions/1616957/how-to-install-a-plugin-in-far-manager)  
10. far2l/far2l/bootstrap/scripts/FarEng.hlf.m4 at master · elfmz/far2l · GitHub, дата последнего обращения: апреля 5, 2026, [https://github.com/elfmz/far2l/blob/master/far2l/bootstrap/scripts/FarEng.hlf.m4](https://github.com/elfmz/far2l/blob/master/far2l/bootstrap/scripts/FarEng.hlf.m4)  
11. FAR Plugins API History \- Far Manager Documentation, дата последнего обращения: апреля 5, 2026, [https://documentation.help/Far-Manager/historyapi.html](https://documentation.help/Far-Manager/historyapi.html)  
12. elfmz/far2l: Linux port of FAR v2 \- GitHub, дата последнего обращения: апреля 5, 2026, [https://github.com/elfmz/far2l](https://github.com/elfmz/far2l)  
13. Plugin System: Building a Cross-Platform C++ Solution \- Pav Creations, дата последнего обращения: апреля 5, 2026, [https://pavcreations.com/plugin-system-building-a-cross-platform-c-solution/](https://pavcreations.com/plugin-system-building-a-cross-platform-c-solution/)  
14. FreshPorts \-- misc/far2l: Port of FAR v2 to Unix-like systems, дата последнего обращения: апреля 5, 2026, [https://www.freshports.org/misc/far2l](https://www.freshports.org/misc/far2l)  
15. Cross Compiling With CMake, дата последнего обращения: апреля 5, 2026, [https://cmake.org/cmake/help/book/mastering-cmake/chapter/Cross%20Compiling%20With%20CMake.html](https://cmake.org/cmake/help/book/mastering-cmake/chapter/Cross%20Compiling%20With%20CMake.html)  
16. \[SOLVED\] If I switch to Linux \- Lazarus forum \- Free Pascal, дата последнего обращения: апреля 5, 2026, [https://forum.lazarus.freepascal.org/index.php?topic=67364.0](https://forum.lazarus.freepascal.org/index.php?topic=67364.0)