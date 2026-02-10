# Velocity — High-Performance Discord Client

[![GitHub](https://img.shields.io/badge/GitHub-lonestill%2Fvelocity--client-blue)](https://github.com/lonestill/velocity-client)

Легковесный, модульный клиент для Discord с упором на максимальную производительность, минимальное потребление RAM и кастомизируемый UI (эстетика Cyberpunk/Fintech).

**Стек:** Rust + **Dioxus** (desktop). Один язык, один рантайм, без отдельного фронтенда.

---

## 1. Общие сведения

| Аспект | Выбор |
|--------|--------|
| **UI + логика** | Rust, Dioxus 0.7 (desktop) |
| **Сеть** | tokio-tungstenite (WebSocket), reqwest (HTTP) |
| **Стили** | CSS в `assets/main.css`, inline-стили в компонентах |

---

## 2. Архитектура приложения

Единое Rust-приложение: UI (Dioxus) и «движок» в одном процессе.

### A. Слой сети и состояния

- **WebSocket** (`tokio-tungstenite`): Discord Gateway v9/v10 (Heartbeat, Identify, READY, GUILD_CREATE, MESSAGE_CREATE).
- **HTTP** (`reqwest`): REST API — отправка сообщений, загрузка истории, профили.
- **State**: `use_signal` в Dioxus для гильдий, каналов, сообщений; фильтрация событий перед обновлением UI.
- **Токен**: OS Keyring (`keyring`), загрузка при старте, форма логина в UI.

### B. UI (Dioxus)

- **Layout**: боковая панель серверов, список каналов, список сообщений.
- **Virtual Scroller**: при необходимости — виртуализация списка сообщений (только видимые + буфер).
- **Стиль**: dark mode, неоновые акценты, glassmorphism.

---

## 3. Зависимости (Cargo.toml)

- `dioxus` (feature `desktop`) — UI и окно.
- `tokio` (features = `["full"]`) — асинхронный рантайм.
- `serde`, `serde_json` — JSON.
- `reqwest` — REST API.
- `tokio-tungstenite` — WebSocket.
- `chrono` — время.
- `anyhow` — ошибки.
- `keyring` — хранение токена в OS Keyring.

---

## 4. Функциональные требования (MVP)

### Этап 1: "The Observer" (только чтение)

| Требование | Описание |
|------------|----------|
| **Login** | Вход по User Token. Токен в OS Keyring, форма логина в Dioxus. |
| **Gateway** | Подключение к Gateway v9/v10, Heartbeat, READY, GUILD_CREATE → список серверов. |
| **Message Rendering** | MESSAGE_CREATE → отрисовка текста, базовый Markdown (жирный, курсив). |

### Этап 2: "The Speaker" (чат)

| Требование | Описание |
|------------|----------|
| **Send Message** | POST `/channels/{id}/messages`. |
| **Channel Switch** | При выборе канала — подгрузка последних 50 сообщений через REST. |

---

## 5. Требования к UI/UX

- **Aesthetic:** тёмная тема, неоновые акценты, glassmorphism на боковых панелях.
- **Performance Metrics (опционально):** в углу — FPS и потребление памяти.

---

## 6. Сборка и запуск

- Установка [Dioxus CLI](https://dioxuslabs.com/learn/0.7/tutorial/new_app/) (опционально):  
  `cargo install dioxus-cli`
- Запуск desktop:  
  `cargo run`  
  или  
  `dx serve --desktop`
- Сборка release:  
  `cargo build --release`

### Auto-update

Приложение проверяет обновления с [GitHub Releases](https://github.com/lonestill/velocity-client/releases). Нажмите **Settings → General → Check for updates**.

---

## 7. Релизы (для разработчика)

**Автоматический релиз:** при пуше тега `v*` (например `v1.0.0`) запускается [GitHub Actions](.github/workflows/release.yml): собирается бинарник под Windows, Linux и macOS и артефакты заливаются в [Releases](https://github.com/lonestill/velocity-client/releases). Имена артефактов соответствуют self-update: `velocity-{version}-{target}.zip` (Windows) или `velocity-{version}-{target}.tar.gz` (Linux/macOS).

Чтобы выкатить релиз:
1. Обновите версию в `Cargo.toml`.
2. Закоммитьте, затем: `git tag v1.0.0 && git push origin v1.0.0`.

Локальные dev-сборки (`/target/`, `/dist/`, `velocity*.zip`, `velocity*.tar.gz` и т.п.) добавлены в `.gitignore` и в репозиторий не попадают.

---

## 8. Структура проекта

```
.
├── Cargo.toml
├── Dioxus.toml
├── assets/
│   └── main.css
└── src/
    ├── main.rs
    ├── app.rs           # корневой компонент, логин / layout
    ├── state.rs         # Guild, Channel, Message, login(), load_token()
    ├── gateway/
    │   └── mod.rs       # WebSocket Gateway (TODO)
    ├── http/
    │   └── mod.rs       # REST API (TODO)
    └── ui/
        ├── mod.rs
        ├── login_form.rs
        ├── layout.rs
        ├── sidebar.rs
        ├── channel_list.rs
        ├── message_list.rs
        └── metrics_overlay.rs
```

---

## 9. План действий (чеклист)

- [x] Каркас: Dioxus desktop, layout, логин, keyring.
- [ ] Gateway: Identify, Heartbeat, READY, GUILD_CREATE, MESSAGE_CREATE → обновление state.
- [ ] REST: загрузка истории канала, отправка сообщения.
- [ ] Виртуализация списка сообщений (при необходимости).
- [ ] Стили: подключение `assets/main.css`, FPS/RAM в углу.

---

*Velocity — один бинарник на Rust, быстрый старт и ~30 МБ RAM.*
