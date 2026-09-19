# Matriz de Suporte a Hardware e Transportes Rapoo

Este documento fornece detalhes técnicos sobre os dispositivos Rapoo suportados, IDs de Fabricante/Produto, transportes de conexão e recursos.

---

## 🖲️ Dispositivos Testados

### 1. Mouse Rapoo MT760 Pro
- **Vendor ID (VID)**: `0x24AE` (ITON Corp. / Rapoo)
- **Product ID (PID)**:
  - `0x186A` (Cabo USB / Receptor Dongle USB 2.4 GHz)
  - `0x4510` (Modo Bluetooth 5.0)
- **Recursos**: 10 Botões, Scroll Lateral do Polegar, Alternância multi-dispositivo (até 4 dispositivos), Sensor óptico de 800 a 4000 DPI.

### 2. Teclado Rapoo E9050L
- **Vendor ID (VID)**: `0x24AE`
- **Product ID (PID)**: `0x1831`
- **Recursos**: Isolado da lógica de remapeamento do mouse para evitar colisão de dispositivos de entrada.

---

## 📊 Matriz de Recursos por Transporte

| Recurso | Cabo USB | Dongle USB 2.4 GHz | Bluetooth |
| :--- | :---: | :---: | :---: |
| **Detecção do Dispositivo** | ✅ Confirmado | ✅ Confirmado | ✅ Confirmado |
| **Remapeamento de Botões (`uinput`)** | ✅ Confirmado | ✅ Confirmado | ✅ Confirmado |
| **Telemetria de Bateria** | ⚠️ N/A (Alimentado pela USB) | ✅ Confirmado (HID/UPower) | ✅ Confirmado (BlueZ D-Bus) |
| **Perfis em Software** | ✅ Confirmado | ✅ Confirmado | ✅ Confirmado |
| **Configuração de Polling Rate** | 🧪 Experimental | 🧪 Experimental | ⚠️ Gerenciado pelo Kernel (~90-133 Hz) |
| **Ajuste de DPI** | 🧪 Experimental | 🧪 Experimental | 🧪 Experimental |

---

## 🔍 Critérios de Desduplicação

Quando o Rapoo MT760 Pro está conectado simultaneamente por Cabo USB e Bluetooth, o agregador do OpenRapoo isola as interfaces com base na hierarquia de tipo de conexão (Cabo USB > Dongle 2.4G > Bluetooth) para evitar exibir cards duplicados na interface.

