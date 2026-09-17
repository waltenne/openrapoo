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
| 2.4 GHz (receptor USB) | `0x186A` | ✅ **CONFIRMADO** | `ITON Corp. Rapoo NearLink Mouse` |
| Bluetooth | não confirmado | ⚠️ Não testado no Linux | — |
| USB-C (com fio) | não confirmado | ⚠️ Não testado no Linux | — |

**Descoberta importante:** O fabricante real do receptor é **ITON Corp.** (não Rapoo diretamente), e o receptor é identificado como "NearLink Mouse" no kernel quando conectado via receptor USB 2.4 GHz. O receptor expõe **3 interfaces HID** sob o mesmo PID `0x186A`:

| Interface | Localização | Tipo | Nó evdev |
|---|---|---|---|
| Mouse (primária) | `input0` | Mouse | `event22` |
| Teclado | `input1` | Keyboard | `event23` |
| Mouse (secundária) | `input1` | Mouse | `event24` |

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
- Status: **não confirmado / requer investigação adicional**
- Se reconhecido, cria nós `event*` via subsistema HID host.

### NearLink

> Status: **não detectado no Linux** — a conexão via receptor USB 2.4 GHz opera através do nó HID genérico do kernel Linux.

---

## 3. Mapeamento Completo de Entradas (CONFIRMADO)

> ✅ Dados confirmados via `/proc/bus/input/devices` + `identify-buttons` em 2026-09-16.

### `/dev/input/event22` — Mouse Principal (input0)

**Tipos suportados:** `EV_SYN`, `EV_KEY`, `EV_REL`, `EV_MSC`

#### Botões (EV_KEY) — 5 botões confirmados

| Código | Nome evdev | Botão físico | Remapeável via software |
|---|---|---|---|
| `0x110` | `BTN_LEFT` | Clique esquerdo | ✅ |
| `0x111` | `BTN_RIGHT` | Clique direito | ✅ |
| `0x112` | `BTN_MIDDLE` | Clique do meio / roda | ✅ |
| `0x113` | `BTN_SIDE` | Botão lateral traseiro (Voltar) | ✅ |
| `0x114` | `BTN_EXTRA` | Botão lateral dianteiro (Avançar) | ✅ |

#### Eixos relativos (EV_REL) — 4 eixos confirmados

| Código | Nome evdev | Função | Status |
|---|---|---|---|
| `REL_X (0)` | `REL_X` | Movimento horizontal | ✅ Passthrough |
| `REL_Y (1)` | `REL_Y` | Movimento vertical | ✅ Passthrough |
| `REL_WHEEL (8)` | `REL_WHEEL` | Scroll vertical | ✅ Remapeável |
| `REL_HWHEEL (6)` | `REL_HWHEEL` | **Scroll lateral** | ✅ Suportado (declarado) |
| `REL_WHEEL_HI_RES (12)` | `REL_WHEEL_HI_RES` | Scroll vertical alta resolução | ✅ Suportado |
| `REL_HWHEEL_HI_RES (11)` | `REL_HWHEEL_HI_RES` | **Scroll lateral alta resolução** | ✅ Suportado |

> **Nota sobre scroll lateral:** O bitmask confirma suporte a `REL_HWHEEL` e `REL_HWHEEL_HI_RES` em `event22`. O hardware declara a capacidade, mas os eventos só chegam ao host quando a roda física é girada. Se não respondeu no teste, pode ser que a roda precise de mais força física, ou os eventos chegam somente via `REL_HWHEEL_HI_RES` (high-res).

---

### `/dev/input/event23` — Interface de Teclado (input1)

**Tipos suportados:** `EV_SYN`, `EV_KEY`, `EV_REL`, `EV_ABS`, `EV_MSC`, `EV_LED`, `EV_REP`

> Esta é a interface mais rica. Com **212 teclas declaradas**, é onde botões especiais do mouse geram eventos de teclado.

**Eixos REL suportados:** `REL_HWHEEL (6)`, `REL_WHEEL_HI_RES (12)`

#### Teclas relevantes declaradas (subset confirmado)

| Código | Nome evdev | Uso típico |
|---|---|---|
| `0x071` | `KEY_MUTE` | Silenciar |
| `0x072` | `KEY_VOLUMEDOWN` | Volume − |
| `0x073` | `KEY_VOLUMEUP` | Volume + |
| `0x074` | `KEY_POWER` | Energia |
| `0x0E2` | `KEY_MUTE` (media) | Silenciar (HID Consumer) |
| `0x0E7` | `KEY_MEDIA` | Abrir player de mídia |
| `0x0E8` | `KEY_BRIGHTNESSDOWN` | Brilho − |
| `0x0E9` | `KEY_BRIGHTNESSUP` | Brilho + |
| `0x0CE` | `KEY_EJECTCD` | Ejetar |
| `0x03B`–`0x058` | `KEY_F1`–`KEY_F12` | Teclas de função |
| `0x01D` | `KEY_LEFTCTRL` | Ctrl |
| `0x02A` | `KEY_LEFTSHIFT` | Shift |
| `0x038` | `KEY_LEFTALT` | Alt |

> **Conclusão:** O botão DPI e o botão de troca de dispositivo provavelmente geram teclas nessa interface. Execute `identify-buttons --device /dev/input/event23` e pressione cada botão especial.

---

### `/dev/input/event24` — Mouse Secundário (input1)

**Tipos suportados:** `EV_SYN`, `EV_ABS`  
**Uso:** Provavelmente para gestos ou touchpad emulado. Sem botões convencionais.

---

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
