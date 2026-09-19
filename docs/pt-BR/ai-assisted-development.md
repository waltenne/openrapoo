# Declaração de Desenvolvimento com Auxílio de IA

O OpenRapoo é desenvolvido com auxílio de ferramentas de inteligência artificial. A IA é utilizada como ferramenta de apoio para pesquisa, implementação, análise, testes e documentação. As decisões técnicas, revisões, validações em hardware e responsabilidade pelo código permanecem sob responsabilidade dos mantenedores do projeto.

---

## 🔍 Como as Ferramentas de IA são Utilizadas no OpenRapoo

1. **Investigação de Hardware e Barramentos**:
   - Auxílio na análise de descritores HID, estruturas de eventos evdev, interfaces D-Bus do UPower e propriedades D-Bus do BlueZ.
2. **Geração e Refatoração de Código**:
   - Geração de código Rust idiomático entre os crates do workspace (`openrapoo-core`, `openrapoo-daemon`, `openrapoo-gui`, `openrapoo-diag`).
   - Construção de componentes de UI em GPUI e catálogos de internacionalização (i18n).
3. **Suíte de Testes Automatizados**:
   - Criação de suítes de testes unitários e de integração (`cargo test --workspace`) cobrindo telemetria, mensagens IPC e persistência de configurações.
4. **CI/CD e Suíte de Empacotamento**:
   - Configuração de workflows do GitHub Actions (`ci.yml`, `release.yml`), leiautes de pacotes Debian `.deb` e scripts de AppImage.
5. **Documentação**:
   - Redação e sincronização de árvores de documentação bilíngue em Inglês e Português do Brasil.

---

## 🛡️ Validação Humana e Garantias de Qualidade

- **Sem Aceitação Cega**: Sugestões de código geradas por IA são rigorosamente avaliadas contra as regras do compilador Rust, `cargo clippy` e revisão manual de código.
- **Validação em Hardware**: Comportamentos em hardware real (botões físicos do Rapoo MT760 Pro, relatórios HID raw, alternância de transportes USB/Bluetooth) são validados em ambiente Linux.
- **Segurança e Permissões**: Scripts críticos de segurança (regras UDev, unidades de serviço systemd, permissões de usuário) são revisados para evitar riscos de elevação de privilégios.

