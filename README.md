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
| Detectar Rapoo MT760 Pro conectado | ✅ |
| Capturar eventos evdev (movimentos, cliques) | ✅ |
| Identificar botões por evento | ✅ |
| Ler HID descriptor (read-only) | ✅ |
| Gerar relatório técnico de diagnóstico | ✅ |

### O que **não** funciona (ainda)

| Funcionalidade | Motivo |
|---|---|
| Ajuste de DPI | Requer protocolo HID proprietário (Fase 8) |
| Configuração na memória do mouse | Protocolo não documentado |
| Remapeamento de botões | Fase 3 (em desenvolvimento) |
| Interface gráfica | Fase 5 (planejada) |
| NearLink no Linux | Sem suporte no kernel Linux |

---

## Instalação (Fase 1 — Ferramenta de diagnóstico)

### Dependências

```bash
# Ubuntu/Debian
sudo apt install libhidapi-dev libudev-dev build-essential

# Fedora/RHEL
sudo dnf install hidapi-devel systemd-devel gcc
```

### Compilar

```bash
git clone https://github.com/openrapoo/openrapoo
cd openrapoo
cargo build --release
```

### Configurar permissões de acesso

```bash
# Instalar regras udev (necessário uma vez)
sudo cp udev/99-openrapoo.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo usermod -aG input $USER
# Reiniciar a sessão para aplicar
```

---

## Uso — CLI de Diagnóstico (`openrapoo-diag`)

### Listar dispositivos Rapoo conectados

```bash
./target/release/openrapoo-diag list-devices
```

Exemplo de saída:
```
Dispositivos Rapoo detectados:
  [1] Rapoo MT760 Pro
      VID: 0x24AE  PID: 0x????
      Conexão: USB (wired)
      evdev:  /dev/input/event5
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
