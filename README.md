# Velocity — High-Performance Discord Client

[![GitHub](https://img.shields.io/badge/GitHub-lonestill%2Fvelocity--client-blue)](https://github.com/lonestill/velocity-client) [![Release](https://github.com/lonestill/velocity-client/actions/workflows/release.yml/badge.svg)](https://github.com/lonestill/velocity-client/actions/workflows/release.yml)

Лёгкий десктопный клиент для Discord: минимум RAM, тёмный интерфейс с неоновыми акцентами.

---

## Скачать

**[Releases](https://github.com/lonestill/velocity-client/releases)** — готовые сборки для Windows, Linux и macOS. Выберите архив под свою ОС, распакуйте и запустите `velocity` (или `velocity.exe` на Windows).

---

## Запуск

- **Windows:** распаковать архив и запустить `velocity.exe`.
- **Linux / macOS:** распаковать архив и запустить `./velocity` (при необходимости: `chmod +x velocity`).

**Linux:** если при запуске появляется ошибка вида `cannot open shared object file: libxdo.so.3` или `libasound.so.2`, установите зависимости (пример для Ubuntu/Debian):

```bash
sudo apt install libxdo3 libasound2 libwebkit2gtk-4.1-0 libgtk-3-0 libglib2.0-0 libayatana-appindicator3-1
```

Токен Discord хранится в системном хранилище (Keyring / Credential Manager).

---

## Обновления

В приложении: **Settings (⚙) → General → Check for updates**. Обновления подтягиваются с GitHub Releases.

---

## Голосовые каналы (опционально)

Поддержка войс-чатов на серверах реализована через библиотеку [Songbird](https://github.com/serenity-rs/songbird). Сборка с голосом требует feature и нативные зависимости (Opus, CMake):

```bash
cargo build --features voice
```

На Windows для сборки с голосом нужен [Opus](https://opus-codec.org/) (например, через vcpkg: `vcpkg install opus`) и CMake. Без feature `voice` клиент собирается и работает как раньше, без кнопок Join/Leave в списке каналов.

---

*Velocity — один бинарник, быстрый старт, ~30 МБ RAM.*
