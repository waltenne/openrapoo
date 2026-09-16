# Contribuindo com o OpenRapoo

Obrigado pelo interesse em contribuir! Leia este guia antes de abrir pull requests.

## Código de Conduta

Este projeto segue o [Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).

## Como contribuir

### Reportar bugs

Abra uma issue com:
- Versão do OpenRapoo
- Versão do kernel Linux (`uname -r`)
- Distro e versão
- Saída do `openrapoo-diag list-devices`
- Saída do `openrapoo-diag generate-report`
- Passos para reproduzir

### Contribuir com dados de hardware

Se você tem um Rapoo MT760 Pro, a contribuição mais valiosa agora é rodar o diagnóstico e compartilhar os dados.

Veja [`docs/log-collection.md`](docs/log-collection.md).

### Pull Requests

1. Fork o repositório
2. Crie uma branch: `git checkout -b feature/minha-funcionalidade`
3. Siga o estilo de código: `cargo fmt` e `cargo clippy`
4. Adicione testes quando aplicável
5. Abra o PR descrevendo o que foi feito

### Regras de segurança para contribuidores

- **Nunca** adicione código que escreva dados no mouse sem confirmação explícita do usuário
- **Nunca** adicione comandos HID não documentados
- Toda operação de escrita deve ter modo de rollback
- Valide todos os inputs para evitar shell injection

## Setup de desenvolvimento

```bash
# Dependências
sudo apt install libhidapi-dev libudev-dev build-essential

# Compilar
cargo build

# Testes
cargo test --workspace

# Linting
cargo clippy --workspace -- -D warnings

# Formatação
cargo fmt --all
```

## Estrutura do repositório

```
crates/
├── openrapoo-core/    # Tipos compartilhados, detecção de dispositivo
├── openrapoo-diag/    # CLI de diagnóstico
└── openrapoo-daemon/  # Daemon de remapeamento (Fase 3)
docs/                  # Documentação técnica
udev/                  # Regras udev
tests/                 # Testes de integração
```

