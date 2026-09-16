# OpenRapoo — Roadmap

## Fase 1 — Investigação e Diagnóstico (atual)

**Objetivo:** Entender o hardware antes de qualquer implementação de alto nível.

- [x] Estrutura do repositório Rust workspace
- [x] Crate `openrapoo-core` com tipos de dispositivo
- [x] Ferramenta CLI `openrapoo-diag`
  - [x] `list-devices` — detectar dispositivos Rapoo por VID `0x24AE`
  - [x] `capture-events` — capturar eventos evdev em tempo real
  - [x] `identify-buttons` — modo interativo para mapear botões
  - [x] `hid-report` — dump read-only do HID descriptor
  - [x] `generate-report` — relatório técnico em Markdown
- [x] Regras udev para acesso sem root
- [x] Documentação técnica do hardware
- [x] Testes automatizados com dispositivo simulado

## Fase 2 — Leitura Completa de Eventos via evdev

**Objetivo:** Mapear todos os botões com eventos + identificar botões silenciosos.

- [ ] Tabela completa de eventos do MT760 Pro
- [ ] Detecção de botões que não geram eventos no Linux
- [ ] Modo "pressione para identificar" na CLI
- [ ] Documentação do mapeamento botão → evento

## Fase 3 — Remapeamento por Software

**Objetivo:** Interceptar eventos do mouse e injetar novos via uinput.

- [ ] Daemon `openrapoo-daemon`
- [ ] Engine de remapeamento: evdev grab → transformação → uinput inject
- [ ] Suporte a ações básicas: teclas, combinações, botões do mouse
- [ ] Systemd user service
- [ ] Comunicação via D-Bus

## Fase 4 — Perfis e Associação a Aplicativos

**Objetivo:** Configurações por perfil com troca automática por aplicativo ativo.

- [ ] Formato JSON de perfil
- [ ] CRUD de perfis
- [ ] Detecção de janela ativa (xdg-activation / AT-SPI2 / sway IPC)
- [ ] Troca automática de perfil por aplicativo

## Fase 5 — Interface Gráfica GTK4

**Objetivo:** GUI moderna e acessível para usuários não-técnicos.

- [ ] Setup GTK4 + libadwaita em Rust (`gtk4-rs`)
- [ ] Tela inicial com mouse detectado + tipo de conexão
- [ ] Representação visual do mouse (SVG interativo)
- [ ] Lista de botões configuráveis
- [ ] Editor de ações
- [ ] Gerenciador de perfis
- [ ] Página de diagnóstico
- [ ] Página de permissões
- [ ] Página de logs
- [ ] Suporte a temas claro/escuro (Adwaita)
- [ ] Internacionalização: pt-BR e en-US (gettext)

## Fase 6 — Serviço em Segundo Plano e Autostart

**Objetivo:** Experiência "instalar e esquecer".

- [ ] Daemon gerenciado pelo systemd --user
- [ ] Autostart XDG (`.config/autostart/`)
- [ ] Notificações via libnotify ao trocar de perfil
- [ ] Tratamento de hotplug (reconectar mouse)

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

