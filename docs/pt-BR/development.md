# Guia de Desenvolvimento do OpenRapoo

Este guia engloba compilação, testes, lints, adição de chaves i18n e empacotamento local do OpenRapoo.

---

## 🛠️ Configuração do Ambiente de Desenvolvimento

Certifique-se de ter o Rust (1.75+) instalado via `rustup`:

```bash
rustup update stable
```

Instale as dependências C de desenvolvimento (Ubuntu/Debian):

```bash
sudo apt install build-essential pkg-config libhidapi-dev libudev-dev libevdev-dev libdbus-1-dev desktop-file-utils
```

---

## 💻 Comandos Cargo Frequentes

```bash
# Verificação de formatação de código
cargo fmt --all -- --check

# Checagem de compilação do workspace
cargo check --workspace

# Lints do Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Execução de todos os testes unitários e de integração
cargo test --workspace

# Compilação de binários release otimizados
cargo build --release --workspace
```

---

## 🌐 Adicionando Novas Chaves de Tradução (i18n)

Todas as traduções da UI são gerenciadas em `crates/openrapoo-gui/src/i18n.rs`. Para adicionar uma nova frase:

1. Adicione uma função auxiliar no domínio correspondente aceitando `lang: Language`:
   ```rust
   pub fn meu_novo_rotulo(lang: Language) -> &'static str {
       match lang {
           Language::English => "My New Label",
           Language::Portuguese => "Meu Novo Rótulo",
       }
   }
   ```
2. Referencie `i18n::meu_novo_rotulo(language)` no seu componente GPUI.
3. Execute `cargo test --workspace` para validar a suíte de testes.

---

## 📦 Execução da Suíte de Empacotamento Local

Para gerar os pacotes `.deb`, `.AppImage` e `.tar.gz` localmente:

```bash
./packaging/build-all.sh
```

Os arquivos finais serão gerados no diretório `dist/`.

