# Как участвовать в разработке Obscur Kit

[English version](CONTRIBUTING.md) · [README](README.ru.md)

Соблюдайте [соглашение о качестве кода](CODE_QUALITY.ru.md) и пишите сообщения
коммитов на английском по [Conventional Commits](COMMIT_CONVENTIONS.ru.md).
Размещайте изменения в соответствующем слое рабочего пространства и сверяйтесь
с [архитектурой крейтов](specs/obscur-kit-2-crate-architecture/).

Перед отправкой pull request запустите проверки из корня рабочего пространства.
Локальные ограничения сборки и обёртка для запуска описаны в [AGENTS.md](AGENTS.md).

```sh
cargo fmt --all --check
cargo test --workspace -j 2
cargo clippy --workspace --all-targets -j 2
cargo run -p xtask -- check
cargo run -p xtask -- consumer
```

Опишите, что изменилось и как вы это проверили. Для платформенных правок укажите
ОС, сервер отображения или композитор, а также устройство или рендерер, если они
влияют на результат.
