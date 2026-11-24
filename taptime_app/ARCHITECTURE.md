# TapTime Android

## Статус

VERIFIED: исходное приложение является шаблоном Tauri/Svelte; идентификатор com.whs31.taptime.app подтверждён прежними Tauri.toml и tauri.conf.json. Документ фиксирует целевую архитектуру, а не завершённость реализации. Android UI, семь виджетов, SDK, сборки и интеграционные проверки требуют отдельного подтверждения результатами выполнения.

## Границы

Android-only: Kotlin, Compose, Material 3, Dynamic Color, Coroutines/Flow, Room, Keystore, WorkManager, RemoteViews, Canvas и Chronometer. Rust SDK экспортируется через UniFFI. ABI: arm64-v8a и x86_64. Сетевой путь: repositories → TapTimeGateway → RustTapTimeGateway → UniFFI → taptime_app/rust → taptime_client → сервер. Общие taptime_core и taptime_schema остаются источниками модели времени и Protobuf.

Разрешены только файлы утверждённого дерева и ограниченный перечень taptime_client: Cargo.toml, src/lib.rs, src/api.rs, src/auth.rs, tests/api_mutations.rs. Остальные модули не изменяются. Перечень файлов и объявлений фиксируется architecture/manifest.toml, DTO и методы — architecture/api-contract.toml. Генерируемые bindings и результаты сборки исходниками не являются. Проверка architecture/check_structure.py не должна изменять зафиксированные контракты.

## Назначения компонентов

Rust: lib.rs регистрирует UniFFI exports; sdk.rs содержит единственный TapTimeSdk, сетевой клиент и сессию; contracts.rs — все DTO и перечисления; mapper.rs — ProtoMapper; clock.rs — WorkClock; timeline.rs — TimelineCalculator; reports.rs — ReportCalculator; errors.rs — ошибки с code, message, retryable и mutation_outcome_unknown.

Application и Activity обеспечивают Android lifecycle; AppContainer собирает зависимости. TapTimeGateway является единственным интерфейсом Android/Rust. RustTapTimeGateway вызывает сгенерированные bindings. SessionRepository управляет авторизацией; WorkRepository — текущим рабочим состоянием; HistoryRepository — днями истории; ReportRepository — отчётами; AccountRepository — профилем и настройками аккаунта. CacheDatabase, CacheEntities и SnapshotDao обеспечивают Room-кеш. TokenStore и KeystoreTokenStore изолируют защищённое хранение токена. AppPreferences хранит локальные предпочтения.

WorkActionCoordinator сериализует мутации из UI и виджетов; WorkClockTicker запускает локальный пересчёт; TimelineMode задаёт режим шкалы. PowerPolicy определяет интервалы с учётом питания; SyncCoordinator координирует синхронизацию; SyncScheduler планирует WorkManager; DashboardSyncWorker выполняет фоновое чтение.

TapTimeApp, TapTimeNavHost и TapTimeTheme отвечают за оболочку, навигацию и тему. Auth/Register, Today, History, Reports, Settings и Profile имеют собственные Screen/ViewModel; ViewModel обращается только к repositories. Components отображают состояние, кольца, шкалы, месячные графики и сессии; EventEditorSheet подготавливает поддерживаемые операции событий; ConfirmActionDialog подтверждает разрушительные действия. Нижняя навигация: Сегодня, История, Отчёты, Настройки; профиль открывается из настроек.

WidgetKind и WidgetState описывают представление; WidgetPreferences хранит настройки по appWidgetId; WidgetRenderer строит RemoteViews; WidgetChartPainter рисует Canvas; WidgetUpdateCoordinator обновляет экземпляры. WidgetConfigurationActivity настраивает экземпляр. WidgetActionReceiver и WidgetActionWorker передают действие общему координатору. Dashboard, DayTimeline, MonthBalance, MonthRing, Compact, Small и Tiny WidgetProvider являются тонкими адаптерами; XML layouts и provider metadata соответствуют каждому типу.

Rust tests проверяют DTO, контракт SDK, время, режимы шкалы, отчёты и безопасность мутаций. Kotlin unit tests проверяют repositories, координаторы, энергополитику и виджеты. Instrumentation tests проверяют авторизацию, главный экран, навигацию, интеграцию виджетов и адаптивность. Результаты запуска фиксируются отдельно; наличие тестового файла не означает успешную проверку.

## Время и серверный контракт

Дата — YYYY-MM-DD; длительности и Unix timestamps — секунды i64; локальное время — секунды от полуночи; timezone — IANA; UUID — строка; отсутствующие значения — Option. Положительный баланс означает переработку, отрицательный — недоработку.

VERIFIED: event.proto хранит UUID и LocalTime, а LocalTime содержит только hour/minute/second. UTC timestamp, offset и DST fold отсутствуют. Поэтому неоднозначный повторяющийся час DST невозможно восстановить из события. SDK не должен выдавать предполагаемый UTC timestamp за подтверждённый факт.

Веб-функции taptime_web/src/lib/dashboard.ts различают work и presence. Work суммирует пары вход/выход; повторный вход заменяет открытый; отрицательная разница обрезается до нуля. Presence — время между первым входом и последним выходом либо текущим временем при открытой сессии. Норма присутствия обычного дня — работа плюс обед. Живой баланс обычного дня основан на presence; для особых дней используется серверный баланс. Итоги исключают beforeStartDate; overtime/undertime агрегируются для обычных дней. Эти правила необходимо сверять с taptime_core и сервером при реализации, отдавая приоритет серверному контракту.

Веб не соединяет события разных дней в единую ночную сессию. Автоматическое соединение через полночь или коррекция DST изменили бы семантику. Проверки обязаны покрывать эти случаи и фиксировать доступную точность, не выдумывая недостающие данные.

WORK_DAY начинается с первого входа и заканчивается ожидаемым завершением по норме с обедом; переработка показывается отдельно. CALENDAR_DAY охватывает 00:00–24:00. Без входа отображается пустое состояние.

## Безопасность и фоновая работа

HTTPS с проверкой сертификатов обязателен. Пароль не хранится; токен SDK находится только в памяти, долговременный токен защищён Android Keystore и исключён из backup. Logout очищает чувствительный кеш и виджеты.

Check In/Out и переключение флага не считаются идемпотентными. После таймаута нельзя автоматически повторять запись. Требуется чтение состояния, а неопределённый результат передаётся явно. Редактирование события не выдаётся за атомарное удаление/создание. Отложенной автоматической отправки пользовательских команд нет.

На переднем плане ориентиры: локальный пересчёт раз в минуту, сеть раз в пять минут. В фоне — WorkManager примерно раз в пятнадцать минут; Battery Saver увеличивает интервал до 30–60 минут. Doze управляется ОС; гарантий минутного фонового исполнения нет. Постоянный foreground service для часов не применяется.

## Известные ограничения реализации

В фиксированном дереве нет app/build.gradle.kts. Допустимая альтернатива — настроить Android-подпроект из корневого build.gradle.kts; добавление отдельного файла требует согласования. Полноценная проверка DST требует учитывать ограничения модели LocalTime. Тестирование подключённого пользовательского телефона, ADB, публикация и Git commit/push требуют отдельного разрешения.
