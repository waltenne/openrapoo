# Como Contribuir para o OpenRapoo

Agradecemos o seu interesse em contribuir com o OpenRapoo! Aceitamos relatórios de bugs, logs de telemetria de hardware, melhorias na documentação e contribuições de código.

---

## 🛠️ Configuração do Ambiente de Desenvolvimento

### Pré-requisitos

Certifique-se de ter o Rust (1.75+) instalado via `rustup` e as bibliotecas C necessárias:

```bash
# Ubuntu / Debian
sudo apt install build-essential pkg-config libhidapi-dev libudev-dev libevdev-dev libdbus-1-dev desktop-file-utils
```

### Verificação e Compilação do Workspace

```bash
# Verificação de formatação
cargo fmt --all -- --check

# Verificação de compilação
cargo check --workspace

# Lints do Clippy
cargo clippy --workspace --all-targets -- -D warnings

# Execução de testes
cargo test --workspace

# Compilação release
cargo build --release --workspace
```

---

## 🐞 Envio de Relatórios de Bug e Telemetria de Hardware

Se você possui um Rapoo MT760 Pro ou outro periférico Rapoo, pode contribuir enviando logs de hardware sem expor informações pessoais:

1. Execute a ferramenta de diagnóstico:
   ```bash
   cargo run --bin openrapoo-diag -- generate-report
   ```
2. O relatório será salvo em `~/.local/share/openrapoo/report-AAAA-MM-DD.md`.
3. Abra uma Issue no GitHub anexando o relatório gerado.

---

## 🤖 Política sobre Uso de Inteligência Artificial

O OpenRapoo aceita Pull Requests desenvolvidos com auxílio de ferramentas de IA. No entanto, certifique-se de que:
1. Todas as alterações foram testadas manualmente em hardware Linux ou validadas com testes unitários.
2. O código segue as convenções do workspace Rust e passa no `cargo clippy -- -D warnings`.
3. Nenhum recurso inexistente ou protocolo inventado pela IA seja apresentado como funcionalidade pronta.

---

## 📜 Processo de Pull Request

1. Faça um Fork do repositório e crie uma branch de recurso (`git checkout -b feature/minha-funcionalidade`).
2. Faça commit de suas alterações com mensagens claras.
3. Confirme se `cargo fmt`, `cargo check`, `cargo clippy` e `cargo test` passam sem erros.
4. Envie sua branch e abra um Pull Request para a branch `main`.

