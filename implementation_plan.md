# OpenRapoo — Plano de Implementação

Aplicativo Linux open source para configurar botões e recursos do mouse **Rapoo MT760 Pro**, com diagnóstico de hardware, remapeamento via software (evdev/uinput), suporte a perfis e futura investigação de protocolo HID proprietário.

---

## Investigação Técnica Prévia

### Hardware: Rapoo MT760 Pro

| Item | Valor / Status |
|---|---|
| **Vendor ID** | `0x24AE` (Shenzhen Rapoo Technology Co., Ltd.) |
| **Product ID** | `0x186A` (confirmado via receptor 2.4 GHz wireless / ITON Corp.) |
| **Sensor** | não confirmado |
| **Botões físicos** | depende do firmware / não confirmado |
| **Entradas programáveis** | depende do firmware / não confirmado |
| **Conectividade** | receptor 2.4 GHz wireless observado (Bluetooth e USB-C com fio requerem investigação adicional) |
| **Dispositivos simultâneos** | não confirmado |
| **Memória onboard** | depende do firmware / não confirmado no Linux |
| **Driver Linux padrão** | `hid-generic` / `usbhid` (plug-and-play para funções básicas) |
| **Protocolo HID proprietário** | não documentado — requer investigação adicional (Fase 8) |

### O que funciona no Linux (estado atual esperado)

| Botão | Status | Observação |
|---|---|---|
| Clique esquerdo | ✅ Funciona | BTN_LEFT |
| Clique direito | ✅ Funciona | BTN_RIGHT |
| Clique do meio | ✅ Funciona | BTN_MIDDLE |
| Scroll vertical | ✅ Funciona | REL_WHEEL |
| Botão Voltar (lateral) | ✅ Funciona | BTN_SIDE / BTN_EXTRA |
| Botão Avançar (lateral) | ✅ Funciona | BTN_FORWARD / BTN_EXTRA |
| Roda lateral | ⚠️ Suportado | REL_HWHEEL (declarado no kernel) |
| Botão DPI | ⚠️ Depende do firmware | Pode gerar evento ou ser silencioso |
| Botão de troca de dispositivo | ❌ Processado pelo firmware | Silencioso no host |
| Ajuste de DPI via software | ❌ Não disponível | Requer protocolo proprietário não documentado |

> [!IMPORTANT]
> O Product ID exato do dispositivo detectado é `0x186A` (Vendor ID `0x24AE`, nome `ITON Corp. Rapoo NearLink Mouse`).

> [!NOTE]
> Suporte a protocolos proprietários e engenharia reversa dependem do comportamento do firmware e testes no Linux.

### Projetos de referência

| Projeto | Relevância |
|---|---|
| **OpenLogi** (AprilNEA/OpenLogi, Rust + GPUI) | Inspiração arquitetural: GUI + Agente + CLI, protocolo HID++, udev rules, systemd |
| **libratbag/Piper** (C + GTK, DBus) | Modelo de separação GUI ↔ daemon ↔ biblioteca HID |
| **xremap** (Rust, evdev + uinput) | Referência para o daemon de remapeamento |
| **input-remapper** (Python, GUI + daemon) | Referência para UX de remapeamento por aplicativo |

---

## Arquitetura do Sistema

```
openrapoo-gui         (GTK4 + libadwaita, Rust)
        │
        │  D-Bus / Unix socket
        ▼
openrapoo-daemon      (Rust, systemd user service)
   ├── device-detector (hidraw + udev)
   ├── event-reader    (evdev)
   ├── remapper        (uinput — injeta eventos virtuais)
   ├── hid-probe       (READ-ONLY hidraw, Fase 8)
   └── profile-manager (JSON / SQLite)

openrapoo-cli         (Rust, diagnóstico e headless)

/etc/udev/rules.d/99-openrapoo.rules  (acesso sem root)
~/.config/openrapoo/                  (perfis, logs)
```

---

## Dois modos de operação (separados explicitamente)

### Modo 1 — Remapeamento por software (MVP)
- Lê eventos via `evdev` de `/dev/input/event*`
- Intercepta com `grab` para não propagar o evento original
- Injeta eventos remapeados via `uinput`
- **Não escreve nada no mouse**
- **Funciona mesmo sem protocolo HID descoberto**

### Modo 2 — Configuração no firmware (Fase 8, opcional)
- Somente após protocolo HID ser **totalmente documentado e validado**
- Requer confirmação explícita do usuário antes de qualquer escrita
- Backup automático antes de alterar
- Modo "seguro" com rollback em caso de falha

---

## Fases de Desenvolvimento

### ✅ Fase 1 — Investigação e Diagnóstico (Entregáveis iniciais)
Objetivo: entender o hardware antes de qualquer implementação de alto nível.

#### Entregáveis
1. **Documento técnico** (`docs/hardware-analysis.md`)
2. **Estrutura do repositório** com Cargo workspace
3. **CLI de diagnóstico** (`openrapoo-diag`)
4. **Detector do Rapoo MT760 Pro** (por VID `0x24AE`)
5. **Testes automatizados** com dispositivos simulados
6. **README** com instruções de coleta de logs
7. **Roadmap** completo
8. **Lista de capacidades** (o que é possível e impossível agora)

### Fase 2 — Leitura de eventos via evdev
- Mapear todos os botões que geram eventos
- Identificar botões silenciosos
- Implementar captura de "pressione para identificar"

### Fase 3 — Remapeamento por software
- Daemon com evdev + uinput
- Regras udev
- Systemd user service

### Fase 4 — Perfis e associação a aplicativos
- Formato JSON de perfil
- Detecção de janela ativa (via D-Bus / xdg-portal)
- Troca automática de perfil

### Fase 5 — Interface GTK4
- GUI em GTK4 + libadwaita
- Representação visual do mouse
- Editor de ações
- Gerenciador de perfis
- Páginas de diagnóstico, permissões, logs

### Fase 6 — Serviço e autostart
- Daemon como systemd user service
- Autostart XDG
- Notificações do sistema

### Fase 7 — Empacotamento
- AppImage
- Flatpak (com portal de permissões)

### Fase 8 — Protocolo HID (opcional/futuro)
- Captura de tráfego USB com Wireshark/usbmon
- Engenharia reversa do software A HUB em VM Windows
- Implementação somente após protocolo totalmente validado

---

## Proposed Changes — Fase 1

### Estrutura do Repositório

#### [NEW] `/home/waltenne/Documents/projects/openrapoo/`

```
openrapoo/
├── Cargo.toml                  # workspace root
├── README.md
├── ROADMAP.md
├── CONTRIBUTING.md
├── LICENSE (GPL-3.0)
├── docs/
│   ├── hardware-analysis.md    # documento técnico
│   ├── capabilities.md         # o que é possível/impossível
│   └── hid-investigation.md    # template para Fase 8
├── crates/
│   ├── openrapoo-core/         # tipos compartilhados, detecção de dispositivo
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── device.rs       # Rapoo device types, VID/PID
│   │       └── error.rs
│   ├── openrapoo-diag/         # CLI de diagnóstico (binário)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── detector.rs     # detecta dispositivos Rapoo
│   │       ├── event_capture.rs # captura evdev
│   │       ├── hid_probe.rs    # lê HID read-only
│   │       └── report.rs       # gera relatório técnico
│   └── openrapoo-daemon/       # stub para Fase 3 (estrutura apenas)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── udev/
│   └── 99-openrapoo.rules
├── tests/
│   └── integration/
│       └── simulated_device.rs
└── .github/
    └── workflows/
        └── ci.yml
```

---

### Crate: `openrapoo-core`

#### [NEW] [`Cargo.toml`](file:///home/waltenne/Documents/projects/openrapoo/crates/openrapoo-core/Cargo.toml)
- Dependências: `thiserror`, `serde`, `serde_json`, `tracing`

#### [NEW] [`src/device.rs`](file:///home/waltenne/Documents/projects/openrapoo/crates/openrapoo-core/src/device.rs)
- `RapooDevice` struct com VID/PID, tipo de conexão, caminho do dispositivo
- `ConnectionType` enum: `Usb`, `TwoPointFourGhz`, `Bluetooth`, `NearLink`, `Unknown`
- `KnownDevice` enum com `RapooMt760Pro` e fallback genérico
- Função `detect_rapoo_devices()` via `/proc/bus/input/devices` + sysfs

---

### Crate: `openrapoo-diag`

#### [NEW] [`src/main.rs`](file:///home/waltenne/Documents/projects/openrapoo/crates/openrapoo-diag/src/main.rs)
- CLI com subcomandos:
  - `list-devices` — lista dispositivos Rapoo conectados
  - `capture-events <device>` — captura e exibe eventos evdev
  - `identify-buttons <device>` — modo interativo "pressione um botão"
  - `hid-report <device>` — dump read-only do HID descriptor e feature reports
  - `generate-report` — gera relatório técnico completo em Markdown/JSON

---

### Udev Rules

#### [NEW] [`udev/99-openrapoo.rules`](file:///home/waltenne/Documents/projects/openrapoo/udev/99-openrapoo.rules)
```
# Rapoo devices - allow access without root
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="24ae", GROUP="input", MODE="0660"
SUBSYSTEM=="input", ATTRS{idVendor}=="24ae", GROUP="input", MODE="0660"
```

---

### Documentação

#### [NEW] [`docs/hardware-analysis.md`](file:///home/waltenne/Documents/projects/openrapoo/docs/hardware-analysis.md)
- Resultado da investigação técnica
- VID/PID confirmados (placeholder para usuário preencher)
- Mapa de botões e eventos evdev esperados
- Limitações conhecidas
- Metodologia de engenharia reversa para Fase 8

#### [NEW] [`docs/capabilities.md`](file:///home/waltenne/Documents/projects/openrapoo/docs/capabilities.md)
- Lista explícita do que É possível configurar (remapeamento por software)
- Lista do que NÃO é possível (ajuste de DPI, memória onboard, firmware)
- Condições para cada limitação ser superada

---

## Requisitos Técnicos

### Dependências Rust

| Crate | Uso |
|---|---|
| `evdev` | Leitura de eventos de input |
| `uinput` | Injeção de eventos virtuais |
| `hidapi` | Comunicação HID de baixo nível |
| `serde` + `serde_json` | Serialização de perfis |
| `tracing` + `tracing-subscriber` | Logs estruturados |
| `clap` | CLI |
| `thiserror` | Tratamento de erros |
| `tokio` | Runtime async para o daemon |
| `nix` | Chamadas de sistema Linux |

### Dependências do sistema

- `libudev-dev` — detecção de dispositivos
- `libhidapi-dev` — acesso HID
- Kernel ≥ 5.4 (para suporte completo a uinput)
- Grupo `input` para acesso sem root

---

## Verificação do Plano

### Testes automatizados
- `cargo test --workspace` — todos os unit tests
- Testes de integração com dispositivo simulado via `uinput`
- CI via GitHub Actions (Ubuntu 22.04 + 24.04)

### Verificação manual
1. Conectar Rapoo MT760 Pro via USB
2. Executar `openrapoo-diag list-devices` → deve detectar o mouse por VID `24AE`
3. Executar `openrapoo-diag capture-events` → deve exibir eventos ao mover/clicar
4. Executar `openrapoo-diag identify-buttons` → deve identificar cada botão pressionado
5. Executar `openrapoo-diag generate-report` → deve gerar relatório em `~/.local/share/openrapoo/report-*.md`

---

## Open Questions

> [!IMPORTANT]
> **Product ID do MT760 Pro**: O PID exato não foi encontrado em bases públicas. A ferramenta de diagnóstico detecta qualquer dispositivo com VID `0x24AE` e pode não saber distinguir variantes (2.4 GHz vs. Bluetooth vs. NearLink) sem o mouse fisicamente conectado para teste.

> [!IMPORTANT]
> **NearLink no Linux**: O protocolo NearLink é proprietário da Huawei e não tem suporte no kernel Linux atual. Se o receptor NearLink é visto como um dispositivo USB genérico 2.4 GHz, tudo bem. Se requer driver específico, não funcionará sem engenharia reversa significativa.

> [!NOTE]
> **Licença**: Proposta GPL-3.0 para garantir copyleft e compatibilidade com o ecossistema Linux. Deseja usar outra licença (MIT, Apache-2.0)?

> [!NOTE]
> **Interface D-Bus vs. Unix socket**: Para comunicação GUI ↔ daemon, D-Bus é o padrão GNOME e permite integração com systemd. Unix socket é mais simples mas menos integrável. A preferência é D-Bus — confirmar?
