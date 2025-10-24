
● Организация работы с Git при портировании на Windows

  Подробная инструкция для solo-разработчика, работающего на двух машинах (Linux + Windows).

  ---
  Общая стратегия

  У вас есть:
  - Linux (основная машина) - /home/holger/github.com/holgertkey/dtree
  - Windows (для портирования) - где-то будет клон репозитория
  - GitHub - центральное хранилище (origin)

  Workflow: Linux ↔ GitHub ↔ Windows

  ---
  Шаг 1: Подготовка на Linux (СЕЙЧАС)

  1.1 Закоммитить текущие изменения

  cd /home/holger/github.com/holgertkey/dtree

  # Посмотреть статус
  git status

  # Добавить новые файлы
  git add WINDOWS_PORTING_GUIDE.md
  git add TESTING_WINDOWS.md
  git add CLAUDE.md  # Обновленный

  # Посмотреть что будет закоммичено
  git status

  # Создать коммит
  git commit -m "docs: add Windows porting guide and update CLAUDE.md for cross-platform

  - Add WINDOWS_PORTING_GUIDE.md with 6-phase porting plan
  - Add TESTING_WINDOWS.md with comprehensive test checklist
  - Update CLAUDE.md with Windows support section
  - Add platform-specific installation instructions
  - Add PowerShell wrapper examples"

  # Проверить что коммит создан
  git log -1

  1.2 Создать ветку для Windows-работы (ОПЦИОНАЛЬНО, но рекомендую)

  # Вариант 1: Работать в main (проще, но рискованнее)
  # - Все изменения сразу в main
  # - Проще для одного разработчика
  # - Но если что-то сломается, main будет нестабильным

  # Вариант 2: Создать feature-ветку (рекомендую)
  # - Изменения изолированы
  # - Можно тестировать отдельно
  # - Мержим в main когда готово

  # Создаем ветку для Windows
  git checkout -b feature/windows-port

  # Проверяем что в правильной ветке
  git branch
  # Должно показать:
  #   main
  # * feature/windows-port

  1.3 Запушить на GitHub

  # Если работаете в main:
  git push origin main

  # Если создали feature-ветку:
  git push -u origin feature/windows-port
  # -u создает связь между локальной и удаленной веткой

  ---
  Шаг 2: Настройка на Windows

  2.1 Установить Git

  # Скачать и установить Git for Windows
  # https://git-scm.com/download/win

  # После установки проверить
  git --version

  2.2 Настроить Git (если еще не делали)

  # Настроить имя и email (такие же как на Linux)
  git config --global user.name "holgertkey"
  git config --global user.email "your-email@example.com"

  # Настроить line endings (ВАЖНО!)
  git config --global core.autocrlf true
  # Это автоматически конвертирует LF ↔ CRLF

  # Проверить настройки
  git config --global --list

  2.3 Клонировать репозиторий

  # Перейти в папку для проектов
  cd C:\Users\YourName\Projects
  # или
  cd D:\Projects

  # Клонировать репозиторий
  git clone https://github.com/holgertkey/dtree.git

  # Перейти в папку
  cd dtree

  # Если работаете в feature-ветке, переключиться на нее
  git checkout feature/windows-port

  # Проверить текущую ветку
  git branch
  # Должно показать:
  #   main
  # * feature/windows-port

  ---
  Шаг 3: Рабочий процесс (Daily Workflow)

  Сценарий 1: Работа на Windows

  # 1. Начинаете работу на Windows
  cd C:\Users\YourName\Projects\dtree

  # 2. ВАЖНО: Сначала получите последние изменения с GitHub
  git pull origin feature/windows-port
  # Это на случай, если вы работали на Linux вчера

  # 3. Проверить статус
  git status

  # 4. Работаете: создаете src/platform.rs, редактируете файлы...

  # 5. Проверить что изменилось
  git status
  git diff  # Посмотреть изменения

  # 6. Тестируете
  cargo build
  cargo test

  # 7. Если все работает, коммитите
  git add src/platform.rs
  git add src/main.rs
  # или добавить все измененные файлы:
  git add -A

  # 8. Создать коммит с понятным сообщением
  git commit -m "feat(platform): add platform-specific module for Windows

  - Create src/platform.rs with Unix/Windows implementations
  - Add open_external_program() function
  - Add is_absolute_path() for cross-platform path detection
  - Update main.rs to use platform module"

  # 9. Запушить на GitHub
  git push origin feature/windows-port

  # Теперь изменения на GitHub и доступны на Linux!

  Сценарий 2: Продолжаете на Linux (на следующий день)

  # 1. Переходите на Linux
  cd /home/holger/github.com/holgertkey/dtree

  # 2. ВАЖНО: Получить изменения с Windows
  git pull origin feature/windows-port

  # 3. Теперь у вас все изменения с Windows!
  # Можете продолжать работу...

  # 4. Работаете: фиксите баг, добавляете тесты...

  # 5. Коммитите
  git add tests/platform_test.rs
  git commit -m "test: add tests for platform module"

  # 6. Пушите
  git push origin feature/windows-port

  # Теперь изменения доступны на Windows!

  Сценарий 3: Возвращаетесь на Windows

  # 1. Снова на Windows
  cd C:\Users\YourName\Projects\dtree

  # 2. Получить изменения с Linux
  git pull origin feature/windows-port

  # 3. Продолжаете работу...

  ---
  Шаг 4: Завершение Windows-порта (когда все готово)

  4.1 Финальное тестирование

  # На Windows
  cargo build --release
  cargo test --release
  # Ручное тестирование по TESTING_WINDOWS.md

  # Все работает? Коммитим
  git add -A
  git commit -m "test: complete Windows testing

  All tests from TESTING_WINDOWS.md passed successfully."
  git push origin feature/windows-port

  4.2 Слияние (Merge) в main

  # На Linux (или можно на Windows)
  cd /home/holger/github.com/holgertkey/dtree

  # Убедиться что все запушено
  git status  # Должно быть чисто

  # Переключиться на main
  git checkout main

  # Получить последние изменения main (на всякий случай)
  git pull origin main

  # Смержить feature-ветку в main
  git merge feature/windows-port

  # Если есть конфликты (обычно нет, если работаете один):
  # - Откройте конфликтные файлы
  # - Разрешите конфликты вручную
  # - git add <файл>
  # - git commit

  # Запушить обновленный main
  git push origin main

  # Удалить feature-ветку (опционально)
  git branch -d feature/windows-port  # Локально
  git push origin --delete feature/windows-port  # На GitHub

  4.3 Обновить Windows на новый main

  # На Windows
  cd C:\Users\YourName\Projects\dtree

  git checkout main
  git pull origin main

  # Теперь Windows тоже на main с полным портом!

  ---
  Шаг 5: Создание Release (опционально)

  # На Linux (после merge в main)
  cd /home/holger/github.com/holgertkey/dtree

  # Обновить версию в Cargo.toml
  # version = "1.1.0"  # Было 1.0.0

  git add Cargo.toml
  git commit -m "chore: bump version to 1.1.0 for Windows support"

  # Создать git tag
  git tag -a v1.1.0 -m "Release v1.1.0 - Windows Support

  - Full Windows compatibility
  - PowerShell wrapper
  - Cross-platform architecture
  - Comprehensive testing suite"

  # Запушить с тегами
  git push origin main --tags

  # На GitHub:
  # - Перейти в Releases
  # - Create new release
  # - Выбрать тег v1.1.0
  # - Добавить описание
  # - Прикрепить binaries (dtree.exe, dtree для Linux)

  ---
  Полезные команды Git

  Просмотр истории

  # История коммитов
  git log

  # Красивая история (одна строка на коммит)
  git log --oneline --graph --all

  # Последний коммит
  git log -1

  # История изменений файла
  git log -p src/main.rs

  Отмена изменений

  # Отменить изменения в файле (НЕ закоммиченные)
  git checkout -- src/main.rs

  # Отменить все незакоммиченные изменения (ОПАСНО!)
  git reset --hard HEAD

  # Отменить последний коммит (НО сохранить изменения)
  git reset --soft HEAD~1

  # Отменить последний коммит (БЕЗ сохранения изменений, ОПАСНО!)
  git reset --hard HEAD~1

  Временно сохранить изменения

  # Сохранить текущие изменения (чтобы переключиться на другую ветку)
  git stash

  # Посмотреть список stash
  git stash list

  # Вернуть сохраненные изменения
  git stash pop

  # Удалить stash
  git stash drop

  Сравнение версий

  # Разница между ветками
  git diff main feature/windows-port

  # Разница между коммитами
  git diff abc123 def456

  # Файлы, которые изменились
  git diff --name-only

  ---
  Рекомендации для solo-разработчика

  ✅ Что НУЖНО делать:

  1. Частые коммиты
  # Коммитьте каждое логическое изменение
  git commit -m "feat: add feature X"
  git commit -m "fix: resolve bug Y"
  git commit -m "test: add tests for Z"
  2. Частые push'ы
  # Пушьте минимум раз в день
  # GitHub = ваш backup!
  git push origin feature/windows-port
  3. Понятные commit messages
  # Хорошо:
  git commit -m "feat(platform): add Windows path handling"
  git commit -m "fix(config): correct Windows config path"
  git commit -m "docs: update README with Windows instructions"

  # Плохо:
  git commit -m "updates"
  git commit -m "fix"
  git commit -m "changes"
  4. git pull перед началом работы
  # ВСЕГДА перед работой:
  git pull origin feature/windows-port
  5. git status часто
  # Проверяйте что происходит:
  git status

  ❌ Чего НЕ нужно делать:

  1. Не коммитить бинарные файлы
  # НЕ добавлять:
  # - target/ (build artifacts)
  # - *.exe (бинарники)
  # - *.o, *.a (object files)

  # Проверить .gitignore:
  cat .gitignore
  2. Не делать force push (без крайней необходимости)
  # ОПАСНО (перезаписывает историю):
  git push --force origin main

  # Только если точно знаете что делаете
  3. Не работать в main напрямую (если важен стабильный код)
  # Лучше:
  git checkout -b feature/something
  # работаете...
  # тестируете...
  git checkout main
  git merge feature/something

  ---
  Типичные ситуации и решения

  Ситуация 1: Забыли сделать git pull, сделали изменения

  # На Windows сделали изменения
  # Пытаетесь запушить:
  git push origin feature/windows-port
  # ERROR: Updates were rejected

  # Решение:
  git pull origin feature/windows-port
  # Git попытается автоматически смержить

  # Если нет конфликтов:
  git push origin feature/windows-port

  # Если есть конфликты:
  # 1. Открыть конфликтные файлы
  # 2. Найти секции с <<<<<<< и >>>>>>>
  # 3. Разрешить конфликт вручную
  # 4. git add <файл>
  # 5. git commit
  # 6. git push origin feature/windows-port

  Ситуация 2: Хотите отменить последний коммит

  # Коммитнули, но ошиблись

  # Вариант 1: Отменить коммит, НО сохранить изменения
  git reset --soft HEAD~1
  # Теперь можете изменить файлы и закоммитить снова

  # Вариант 2: Отменить коммит И изменения (ОПАСНО!)
  git reset --hard HEAD~1
  # ВСЕ изменения потеряны!

  # Если уже запушили:
  # НЕ РЕКОМЕНДУЕТСЯ, но можно:
  git push --force origin feature/windows-port
  # ВНИМАНИЕ: force push = плохо, используйте редко

  Ситуация 3: Случайно изменили файл, хотите вернуть

  # Изменили src/main.rs, хотите вернуть как было

  git checkout -- src/main.rs
  # Файл вернулся к последнему коммиту

  Ситуация 4: Хотите посмотреть что изменилось

  # Что изменилось в файлах (не закоммичено)
  git diff

  # Что будет закоммичено (после git add)
  git diff --staged

  # Изменения в конкретном файле
  git diff src/main.rs

  ---
  Структура коммитов для Windows-порта

  Примерный план коммитов:

  # Фаза 1: Подготовка
  git commit -m "docs: add Windows porting guide"

  # Фаза 2: Платформенный код
  git commit -m "feat(platform): create platform-specific module"
  git commit -m "feat(platform): implement Windows external program handling"
  git commit -m "feat(platform): add Windows path detection"
  git commit -m "refactor(main): use platform module instead of Unix shell"
  git commit -m "feat(config): add Windows default applications"

  # Фаза 3: Shell интеграция
  git commit -m "feat(windows): add PowerShell wrapper script"
  git commit -m "feat(windows): add binary installation script"

  # Фаза 4: Тестирование
  git commit -m "test(windows): add platform-specific tests"
  git commit -m "fix(windows): resolve long path issues"
  git commit -m "fix(windows): handle Unicode in paths"

  # Фаза 5: Документация
  git commit -m "docs: update README with Windows instructions"
  git commit -m "docs: update CLAUDE.md with cross-platform info"
  git commit -m "docs: create CHANGELOG for v1.1.0"

  # Фаза 6: CI/CD
  git commit -m "ci: add Windows build workflow"
  git commit -m "ci: enable cross-platform testing"

  ---
  Чек-лист перед push

  Перед каждым git push:

  - git status - проверить что добавлено
  - cargo build - код компилируется
  - cargo test - тесты проходят
  - git log -1 - проверить сообщение коммита
  - Commit message понятный и описательный
  - Не добавлены лишние файлы (бинарники, временные файлы)

  ---
  Шпаргалка команд

  # === ЕЖЕДНЕВНАЯ РАБОТА ===

  # Начало дня
  git pull origin feature/windows-port

  # Работа
  # ... редактируете файлы ...

  # Проверка
  git status
  git diff

  # Коммит
  git add src/platform.rs
  git commit -m "feat: add platform module"

  # Отправка
  git push origin feature/windows-port

  # === ПЕРЕКЛЮЧЕНИЕ ВЕТОК ===

  # Создать ветку
  git checkout -b feature/new-feature

  # Переключиться на существующую
  git checkout main

  # Список веток
  git branch

  # === ИСТОРИЯ ===

  # Просмотр
  git log --oneline
  git log --oneline --graph --all

  # === ОТМЕНА ===

  # Отменить изменения файла
  git checkout -- filename

  # Отменить последний коммит (сохранить изменения)
  git reset --soft HEAD~1

  # === СИНХРОНИЗАЦИЯ ===

  # Получить изменения
  git pull origin branch-name

  # Отправить изменения
  git push origin branch-name

  # Обновить список веток
  git fetch origin

  # === ИНФОРМАЦИЯ ===

  # Статус
  git status

  # Разница
  git diff
  git diff --staged

  # Удаленные репозитории
  git remote -v

  ---
  Итоговый workflow для вас

  На Linux (сейчас):
  git add WINDOWS_PORTING_GUIDE.md TESTING_WINDOWS.md CLAUDE.md
  git commit -m "docs: add Windows porting documentation"
  git checkout -b feature/windows-port
  git push -u origin feature/windows-port

  На Windows (после клонирования):
  git clone https://github.com/holgertkey/dtree.git
  cd dtree
  git checkout feature/windows-port
  # Работаете...
  git add -A
  git commit -m "feat(platform): implement Windows support"
  git push origin feature/windows-port

  Обратно на Linux:
  git pull origin feature/windows-port
  # Продолжаете работу...

  Когда все готово:
  git checkout main
  git merge feature/windows-port
  git push origin main
  git tag -a v1.1.0 -m "Windows support"
  git push origin main --tags
