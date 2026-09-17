# OpenRapoo

> **Status:** 🔬 Fase 1 — Investigação de Hardware e Diagnóstico

Linux open source configuration tool for the **Rapoo MT760 Pro** mouse.
Inspired by [OpenLogi](https://github.com/AprilNEA/OpenLogi).

---

## ⚠️ Estado Atual

O OpenRapoo está na **Fase 1 de desenvolvimento**: investigação do hardware e ferramenta de diagnóstico. **A interface gráfica ainda não existe.** O objetivo atual é mapear quais botões e recursos do mouse são acessíveis no Linux antes de implementar qualquer funcionalidade de configuração.

### O que funciona agora

| Funcionalidade | Status |
|---|---|
| Detectar Rapoo MT760 Pro conectado (VID `0x24AE`, PID `0x186A`) | ✅ |
| Capturar eventos evdev (movimentos, cliques) | ✅ |
| Identificar botões por evento | ✅ |
| Remapeamento por software (`evdev` + `uinput`) | ✅ |
| Interface Gráfica GTK4 | ✅ |
| Gerar relatório técnico de diagnóstico | ✅ |

---

## Uso — Interface Gráfica e CLI

### Interface Gráfica (GTK4)

```bash
cargo run --bin openrapoo-gui --features gtk
```

### Diagnóstico via CLI (`openrapoo-gui` / `openrapoo-diag`)

```bash
# Diagnóstico de permissões e dispositivo
cargo run --bin openrapoo-gui -- --diagnose

# Listar dispositivos Rapoo detectados
cargo run --bin openrapoo-gui -- --list-devices

# Instalar regras udev
sudo cargo run --bin openrapoo-gui -- --install-udev
```

Exemplo de saída de detecção:
```
Dispositivos Rapoo detectados:
  [1] ITON Corp. Rapoo NearLink Mouse
      VID: 0x24AE  PID: 0x186A
      Conexão: receptor 2.4 GHz wireless
      evdev:  /dev/input/event22
      hidraw: /dev/hidraw2
```

### Capturar eventos em tempo real

```bash
./target/release/openrapoo-diag capture-events --device /dev/input/event5
```

### Identificar botões interativamente

```bash
./target/release/openrapoo-diag identify-buttons --device /dev/input/event5
```

Modo interativo: pressione cada botão do mouse para ver seu código de evento.

### Inspecionar HID descriptor (read-only)

```bash
./target/release/openrapoo-diag hid-report --device /dev/hidraw2
```

### Gerar relatório técnico completo

```bash
./target/release/openrapoo-diag generate-report
```

O relatório é salvo em `~/.local/share/openrapoo/report-YYYY-MM-DD.md`.

---

## Como contribuir com dados de diagnóstico

Se você tem um Rapoo MT760 Pro e quer ajudar o projeto:

1. Compile o `openrapoo-diag`
2. Execute `openrapoo-diag generate-report`
3. Abra uma issue no GitHub anexando o relatório gerado
4. O relatório **não contém dados pessoais** — apenas informações do hardware

Veja [`docs/log-collection.md`](docs/log-collection.md) para instruções detalhadas.

---

## Arquitetura

```
openrapoo-gui         (GTK4 + libadwaita) — Fase 5
        │
        │  D-Bus
        ▼
openrapoo-daemon      (systemd user service) — Fase 3
   ├── event-reader   (evdev)
   ├── remapper       (uinput)
   └── profile-mgr    (JSON)

openrapoo-diag        (CLI diagnóstico) — Fase 1 ✅
openrapoo-core        (biblioteca compartilhada)
```

---

## Roadmap

Veja [`ROADMAP.md`](ROADMAP.md).

---

## Licença

GPL-3.0-or-later. Veja [`LICENSE`](LICENSE).

---

## Aviso Legal

Este projeto **não é afiliado, endossado ou patrocinado** pela Rapoo Technology Co., Ltd.
O nome "Rapoo" e "MT760 Pro" são marcas registradas de seus respectivos proprietários.

