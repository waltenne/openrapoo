# OpenRapoo — Roadmap

## Fase 1 — Investigação e Diagnóstico (Concluída ✅)

**Objetivo:** Entender o hardware antes de qualquer implementação de alto nível.

- [x] Estrutura do repositório Rust workspace (`openrapoo-core`, `openrapoo-diag`, `openrapoo-daemon`, `openrapoo-gui`)
- [x] Crate `openrapoo-core` com tipos de dispositivo e detecção por VID `0x24AE`
- [x] Ferramenta CLI `openrapoo-diag`
  - [x] `list-devices` — detectar dispositivos Rapoo por VID `0x24AE`
  - [x] `capture-events` — capturar eventos evdev em tempo real
  - [x] `identify-buttons` — modo interativo para mapear botões e eixos relativos
  - [x] `hid-report` — dump read-only do HID descriptor
  - [x] `generate-report` — relatório técnico em Markdown
- [x] Regras udev (`99-openrapoo.rules`) para acesso sem root
- [x] Documentação técnica do hardware em `docs/hardware-analysis.md`
- [x] Testes automatizados de detecção e serialização

## Fase 2 — Leitura Completa de Eventos via evdev (Concluída ✅)

**Objetivo:** Mapear todos os botões com eventos + identificar botões silenciosos.

- [x] Tabela completa de eventos do MT760 Pro (VID `0x24AE`, PID `0x186A`)
- [x] Identificação das 3 interfaces HID (`event22` mouse, `event23` teclado, `event24` mouse sec)
- [x] Modo interativo de identificação de botões e eixos relativos em `openrapoo-diag`
- [x] Documentação do mapeamento em `docs/hardware-analysis.md`

## Fase 3 — Remapeamento por Software (Concluída ✅)

**Objetivo:** Interceptar eventos do mouse e injetar novos via uinput.

- [x] Daemon `openrapoo-daemon` em Rust
- [x] Engine de remapeamento: `evdev grab` → transformação de ações → injeção `uinput`
- [x] Suporte a ações: cliques de mouse, teclas, combinações (`Ctrl+C`, `Ctrl+V`, `Alt+F4`), controle de mídia, execução de comandos seguros
- [x] Modo seguro `--dry-run` para testes e depuração
- [x] Tratamento de sinais de encerramento limpo (`SIGINT`/`SIGTERM`)

## Fase 4 — Perfis e Associação a Aplicativos (Concluída ✅)

**Objetivo:** Configurações por perfil com persistência e fallback.

- [x] Formato JSON de perfil e carregador `ProfileStore` em `~/.config/openrapoo/profiles.json`
- [x] Suporte a associação de perfis por aplicativo (`app_association`)
- [x] Perfil padrão inteligente Passthrough
- [x] Validação de comandos customizados contra shell injection

## Fase 5 — Interface Gráfica GTK4 (Concluída ✅)

**Objetivo:** GUI moderna e acessível para usuários não-técnicos.

- [x] Crate `openrapoo-gui` no workspace Cargo
- [x] Tela inicial com mouse detectado + tipo de conexão + status de permissões
- [x] Lista de botões configuráveis e editor de ações por categoria
- [x] Gerenciador de perfis (CRUD de perfis em JSON e associação com apps)
- [x] Páginas de diagnóstico, verificação de permissões udev/grupo `input` e visualização de logs
- [x] Suporte a internacionalização: Português (pt-BR) e Inglês (en-US)
- [x] Flag opcional `gtk` para compilação cruzada universal

## Fase 6 — Serviço em Segundo Plano e Autostart (Concluída ✅)

**Objetivo:** Experiência "instalar e esquecer".

- [x] Daemon gerenciado pelo systemd user service (`systemd/openrapoo-daemon.service`)
- [x] Arquivo Autostart XDG (`autostart/openrapoo-autostart.desktop`)
- [x] Script de instalação automatizada (`scripts/install-service.sh`)
- [x] Tratamento de hotplug e reconexão automática ao desconectar/reconectar o mouse
- [x] Liberação graciosa de `grab()` de nós evdev em desligamento de serviço

## Fase 7 — Empacotamento

**Objetivo:** Distribuição fácil para usuários finais.

- [ ] AppImage via `cargo-appimage` ou `linuxdeploy`
- [ ] Flatpak manifest (`io.github.openrapoo.OpenRapoo.yaml`)
- [ ] Publicação no Flathub
- [ ] AUR (Arch Linux User Repository)

## Fase 8 — Protocolo HID Proprietário (Opcional)

**Objetivo:** Ajuste de DPI, configuração de memória onboard — **somente se protocolo validado**.

- [ ] Captura de tráfego USB com `usbmon` + Wireshark
- [ ] Análise do software A HUB em VM Windows
- [ ] Documentação do protocolo
- [ ] Implementação com modo seguro e rollback
- [ ] Testes exaustivos antes de qualquer escrita no dispositivo

---

> **Nota:** As fases 1–7 não requerem o protocolo HID proprietário. O remapeamento por software (Fases 2–6) funciona independentemente.
