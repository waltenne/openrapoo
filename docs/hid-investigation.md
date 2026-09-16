# Investigação do Protocolo HID Proprietário — Fase 8

> **Status:** Planejado (não iniciado)  
> **Pré-requisito:** Fases 1–7 concluídas  
> **Nível de risco:** Alto — operações de escrita no firmware do dispositivo  

---

## Objetivo

Descobrir e documentar o protocolo HID proprietário do Rapoo MT760 Pro para permitir:
- Ajuste de DPI diretamente pelo Linux
- Leitura e gravação de configurações na memória onboard
- Configuração de botões que não geram eventos evdev

---

## Metodologia de Engenharia Reversa

### Passo 1: Preparar o ambiente de captura

```bash
# Carregar módulo usbmon
sudo modprobe usbmon

# Verificar que está carregado
ls /sys/kernel/debug/usb/

# Instalar Wireshark
sudo apt install wireshark
sudo usermod -aG wireshark $USER
```

### Passo 2: Identificar o barramento USB

```bash
# Ver qual barramento (usbmonX) corresponde ao mouse
lsusb -t
# Procure o mouse Rapoo e anote o número do barramento
```

### Passo 3: Capturar tráfego no Windows (VM)

1. Instale uma VM Windows (VirtualBox ou QEMU)
2. Passe o receptor USB para a VM
3. Instale o software A HUB da Rapoo na VM
4. No host Linux, inicie a captura: `sudo wireshark -i usbmonX`
5. Na VM, altere configurações (DPI, botões, etc.)
6. Salve o arquivo `.pcap`

### Passo 4: Analisar os pacotes

1. Abra o `.pcap` no Wireshark
2. Filtre por: `usb.transfer_type == 0x01` (Interrupt) e `usb.data_len > 0`
3. Compare pacotes antes e após cada mudança de configuração
4. Identifique padrões (report ID, bytes que mudam, checksum)

---

## Regras de Segurança para Fase 8

> ⚠️ **NUNCA faça o seguinte sem confirmação explícita do usuário:**
> - Enviar HID Output Reports não documentados
> - Enviar HID Feature Reports de escrita (SET_REPORT)
> - Modificar firmware do dispositivo

> ✅ **Sempre inclua:**
> - Backup completo do estado atual antes de qualquer escrita
> - Modo "dry run" que mostra o que seria enviado sem enviar
> - Confirmação do usuário com o hexdump exato do comando
> - Possibilidade de rollback imediato
> - Documentação de cada comando descoberto antes de implementá-lo

---

## Formato de Documentação do Protocolo

Quando um comando for descoberto, documentar assim:

```
## Comando: Set DPI

Report ID: 0x04 (Feature Report)
Direção:   SET_REPORT (host → dispositivo)
Tamanho:   8 bytes

Estrutura:
  Byte 0: Report ID = 0x04
  Byte 1: Command = 0x01 (Set DPI)
  Byte 2: DPI low byte (ex: 0xE0 para 800 DPI → 0x0320 = 800)
  Byte 3: DPI high byte
  Bytes 4-7: padding (0x00)

Exemplo (800 DPI):
  04 01 20 03 00 00 00 00

Resposta esperada:
  04 00 (ACK)

Testado em:
  - Hardware: MT760 Pro, FW 1.0.3
  - Data: 2026-09-XX
  - Testado por: @username
```

---

## Ferramentas Úteis

- [Wireshark com USBPcap](https://www.wireshark.org/)
- [usbmon kernel module](https://www.kernel.org/doc/Documentation/usb/usbmon.rst)
- [hidviz](https://github.com/ondrejbudai/hidviz) — visualizador de HID reports
- [python-hid](https://github.com/apmorton/pyhidapi) — prototipar em Python antes de Rust
- [hid-tools](https://gitlab.freedesktop.org/libevdev/hid-tools) — análise de HID descriptors

