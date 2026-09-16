# Análise Técnica de Hardware — Rapoo MT760 Pro no Linux

> **Versão:** 0.1 (Fase 1 — investigação inicial)  
> **Status:** Em progresso — dados de hardware ainda precisam de confirmação com o dispositivo físico.

---

## 1. Identificação do Dispositivo

### Vendor ID

| Campo | Valor |
|---|---|
| **Vendor ID (VID)** | `0x24AE` |
| **Fabricante** | Shenzhen Rapoo Technology Co., Ltd. |
| **Fonte** | linux-usb.org, devicehunt.com |

### Product ID

> ✅ **PID confirmado por execução do `openrapoo-diag` no hardware real.**

| Modo de conexão | PID | Status | Nome no kernel |
|---|---|---|---|
| NearLink/2.4 GHz (receptor USB) | `0x186A` | ✅ **CONFIRMADO** | `ITON Corp. Rapoo NearLink Mouse` |
| Bluetooth 5.0 | desconhecido | ⚠️ Não testado | — |
| USB-C (com fio) | desconhecido | ⚠️ Não testado | — |

**Descoberta importante:** O fabricante real do receptor é **ITON Corp.** (não Rapoo diretamente), e o receptor é identificado como "NearLink Mouse" mesmo quando conectado via 2.4 GHz regular. O receptor expõe **3 interfaces HID** sob o mesmo PID `0x186A`:

| Interface | Localização | Tipo | Nó evdev |
|---|---|---|---|
| Mouse (primária) | `input0` | Mouse | `event22` |
| Teclado | `input1` | Keyboard | `event23` |
| Mouse (secundária) | `input1` | Mouse | `event24` |

A interface de **Keyboard** (`event23`) é especialmente interessante: indica que botões adicionais do mouse podem gerar eventos de **teclado**, não apenas eventos de mouse. Isso é comum em mouses com botões de mídia ou atalhos.

### HID devices em `/sys/bus/hid/devices`

```
0003:24AE:186A.000D
0003:24AE:186A.000E
```

### Como confirmar o PID para Bluetooth e USB-C

```bash
# Conecte via Bluetooth e execute:
lsusb | grep -i '24ae'
# ou
openrapoo-diag list-devices

# Repita para cada modo de conexão
```

---

## 2. Interfaces no Linux

### USB / 2.4 GHz

O receptor USB 2.4 GHz é reconhecido como um dispositivo HID padrão. O kernel carrega:
- Driver `usbhid` → `hid-generic`
- Cria nós em `/dev/input/event*` (via subsistema `evdev`)
- Cria nó em `/dev/hidraw*` (acesso HID bruto)

### Bluetooth

Quando conectado via Bluetooth:
- Aparece como dispositivo `/sys/bus/hid/devices/0005:24AE:XXXX.*`
- Bus ID: `0x0005` (BT_HOST_DEVICE)
- Cria nós `event*` da mesma forma
- O nó `hidraw` também está disponível

### NearLink

> ❌ **NearLink é proprietário da Huawei e não tem suporte no kernel Linux.**  
> O receptor NearLink pode aparecer como dispositivo USB HID genérico,  
> mas funcionalidades avançadas requerem driver proprietário.  
> Status: **desconhecido** — precisa de teste com hardware real.

### USB-C Wired

Quando conectado via cabo USB-C:
- Funciona como dispositivo USB HID padrão
- Potencialmente com polling rate mais alto (até 2000 Hz, mas o kernel limita)
- Mesmo comportamento que o receptor 2.4 GHz no nível de driver

---

## 3. Mapeamento de Botões

### Botões em `/dev/input/event22` — confirmados com hardware real

> ✅ Testado em 2026-09-16 com `openrapoo-diag identify-buttons --device /dev/input/event22`

| Botão físico | Evento evdev | Código | Status |
|---|---|---|---|
| Clique esquerdo | `BTN_LEFT` | `0x110` | ✅ **CONFIRMADO** |
| Clique direito | `BTN_RIGHT` | `0x111` | ✅ **CONFIRMADO** |
| Clique do meio (roda) | `BTN_MIDDLE` | `0x112` | ✅ **CONFIRMADO** |
| Botão lateral traseiro | `BTN_SIDE` | `0x113` | ✅ **CONFIRMADO** |
| Botão lateral dianteiro | `BTN_EXTRA` | `0x114` | ✅ **CONFIRMADO** |
| Roda de rolagem vertical | `REL_WHEEL` | `EV_REL 0x08` | ✅ Padrão HID (não testado via identify) |
| **Roda lateral (horizontal)** | `REL_HWHEEL`? | `EV_REL 0x06`? | ⚠️ **Não respondeu** — ver nota abaixo |

> **Nota sobre o scroll lateral:** O `identify-buttons` captura apenas eventos `EV_KEY` (botões). O scroll lateral gera eventos `EV_REL` (eixo relativo), que são invisíveis para esse modo. Use `capture-events` e gire a roda lateral para verificar se gera `REL_HWHEEL` ou se é silencioso.

### Interface `/dev/input/event23` (Keyboard) — **ainda não testada**

Esta interface pode conter os botões:
- DPI (ajuste de sensibilidade)
- Troca de dispositivo
- Botões de mídia (se houver)
- Botão extra customizável

> **Ação necessária:** Execute `identify-buttons --device /dev/input/event23` e pressione cada botão especial do mouse.

### Botões que provavelmente NÃO geram eventos

| Botão | Motivo |
|---|---|
| Troca de dispositivo | Processado pelo firmware/receptor; não há razão para enviar evento ao host |
| DPI | Aguardando teste em `event23` |

---

## 4. Protocolo HID

### Modo padrão (confirmado)

O mouse usa o protocolo HID Boot Protocol / Report Protocol padrão para:
- Movimentos do cursor (HID Usage Page: Generic Desktop, Usage: Mouse)
- Botões padrão (L/R/Meio, Back, Forward)
- Scroll

### Protocolo proprietário (a investigar — Fase 8)

Para funcionalidades avançadas (DPI, LEDs, memória onboard), o software oficial
"A HUB" da Rapoo provavelmente usa:
- HID Feature Reports (via GET_REPORT / SET_REPORT)
- Ou HID Output Reports com comandos proprietários
- Possivelmente via uma interface USB de "Vendor Defined" (Usage Page: `0xFF00`–`0xFFFF`)

> ⚠️ **Nenhum documento público descreve este protocolo.**  
> A investigação na Fase 8 será feita por análise de tráfego USB com Wireshark/usbmon,  
> usando o software A HUB em uma VM Windows.

---

## 5. Memória Onboard

O MT760 Pro possui memória onboard para armazenar configurações.  
Na prática para o Linux isso significa:

- Configurações feitas no Windows com o A HUB **persistem** mesmo no Linux
- Sem o protocolo HID descoberto, não é possível ler ou gravar essas configurações pelo OpenRapoo
- O remapeamento por software (Fase 3) funciona **independentemente** da memória onboard

---

## 6. Diagnóstico com Ferramentas Padrão

```bash
# Verificar se o mouse é detectado
lsusb | grep -i rapoo
lsusb | grep 24ae

# Ver interfaces de entrada
cat /proc/bus/input/devices | grep -A 10 -i rapoo

# Listar nós de evento
ls -la /dev/input/event*

# Listar nós hidraw
ls -la /dev/hidraw*

# Ver HID report descriptor
hexdump -C /sys/bus/hid/devices/0003:24AE:XXXX.YYYY/report_descriptor

# Monitorar eventos em tempo real
sudo evtest /dev/input/eventX

# Listar capabilities do dispositivo
evemu-describe /dev/input/eventX
```

---

## 7. Drivers do Kernel Carregados

```
$ lsmod | grep hid
hid_generic        16384  0
usbhid             53248  0
hid               135168  2 hid_generic,usbhid
```

Nenhum driver específico para Rapoo existe no kernel mainline.

---

## 8. Limitações Conhecidas

1. **Product ID não confirmado**: Os PIDs no código são placeholders.
2. **NearLink sem suporte**: O protocolo NearLink não tem driver no kernel Linux.
3. **Protocolo HID proprietário não documentado**: DPI e memória onboard requerem engenharia reversa.
4. **Polling rate limitado no Bluetooth**: O Bluetooth opera a 125 Hz, não 2000 Hz.
5. **Botões silenciosos**: Troca de dispositivo e possivelmente DPI não geram eventos evdev.

---

## 9. Próximos Passos de Investigação

1. **Confirmar PIDs reais**: Precisamos de usuários com o hardware para rodar `lsusb` e `openrapoo-diag list-devices`.
2. **Mapear todos os botões**: Executar `openrapoo-diag identify-buttons` e documentar todos os códigos.
3. **Verificar roda horizontal**: Confirmar se gera `REL_HWHEEL`.
4. **Analisar HID descriptor**: Executar `openrapoo-diag hid-report` e inspecionar as collections.
5. **Fase 8 (futuro)**: Configurar VM Windows com A HUB + Wireshark para capturar tráfego.

---

## Referências

- [HID Usage Tables 1.3](https://usb.org/document-library/hid-usage-tables-13)
- [Linux Input Subsystem — kernel docs](https://www.kernel.org/doc/html/latest/input/)
- [evdev event codes](https://www.kernel.org/doc/html/latest/input/event-codes.html)
- [hidraw documentation](https://www.kernel.org/doc/Documentation/hid/hidraw.rst)
- [libratbag architecture](https://github.com/libratbag/libratbag)
