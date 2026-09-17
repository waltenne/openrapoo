# O que é possível e o que não é — OpenRapoo

> **Fase atual:** 1 (Diagnóstico)  
> Esta lista será atualizada conforme cada fase for implementada.

---

## ✅ O que É possível agora (Fase 1)

| Funcionalidade | Como | Requisito |
|---|---|---|
| Detectar o Rapoo MT760 Pro conectado | Scan de `/proc/bus/input/devices` por VID `0x24AE` | Grupo `input` |
| Listar todos os dispositivos Rapoo | `openrapoo-diag list-devices` | Grupo `input` |
| Capturar eventos de movimento e clique | `openrapoo-diag capture-events` | Grupo `input` |
| Identificar botões por evento evdev | `openrapoo-diag identify-buttons` | Grupo `input` |
| Ler HID Report Descriptor | `openrapoo-diag hid-report` | Grupo `input` |
| Gerar relatório técnico de diagnóstico | `openrapoo-diag generate-report` | Nenhum |

---

## 🚧 O que será possível em fases futuras

### Fase 3 — Remapeamento por Software

| Funcionalidade | Limitação |
|---|---|
| Reatribuir botões que geram eventos evdev | Requer daemon + uinput |
| Executar comandos ao pressionar um botão | Requer daemon |
| Simular teclas ao pressionar botão do mouse | Requer daemon + uinput |
| Desabilitar um botão | Requer daemon |
| Controle de mídia via botão | Requer daemon |
| Troca de área de trabalho | Requer daemon + D-Bus |

### Fase 4 — Perfis

| Funcionalidade | Limitação |
|---|---|
| Criar e editar perfis | Requer Fase 3 |
| Trocar perfil automaticamente por aplicativo | Requer Fase 3 + detecção de janela ativa |
| Importar/exportar perfis | Requer Fase 3 |

### Fase 5 — Interface Gráfica

| Funcionalidade | Limitação |
|---|---|
| GUI GTK4 para configurar botões | Requer Fases 3 e 4 |
| Representação visual do mouse | Requer Fase 5 |
| Tema claro/escuro | Requer Fase 5 |

---

## ❌ O que NÃO é possível (estado atual)

### Sem solução pelo OpenRapoo (restrição de hardware/protocolo)

| Funcionalidade | Motivo | Condição para mudar |
|---|---|---|
| Ajustar DPI no Linux | Requer protocolo HID proprietário não documentado | Fase 8: investigação adicional |
| Configurar memória onboard do mouse | depende do firmware / não confirmado | Fase 8: investigação adicional |
| Remapear botões que não geram eventos evdev | Eventos silenciosos não chegam ao sistema operacional | Fase 8: possível se houver comando HID |
| Usar NearLink no Linux | não detectado no Linux (opera via receptor USB 2.4 GHz) | Requer investigação adicional |
| Configurar iluminação (se houver) | não confirmado | Fase 8 |

### Botões silenciosos ou tratados pelo firmware

| Botão | Status | Alternativa |
|---|---|---|
| Troca de dispositivo | Processado pelo firmware | Nenhuma — é intencional no hardware |
| DPI | Depende do firmware | Verificar com `identify-buttons` |

---

## ⚠️ O que ainda precisa de investigação adicional

| Questão | Como descobrir |
|---|---|
| Botão DPI gera evento em todos os modos? | `identify-buttons` |
| Roda lateral gera `REL_HWHEEL`? | `capture-events` |
| Comandos HID proprietários para DPI | `hid-report` / captura USBmon |

---

## Como ajudar

Se você tem um Rapoo MT760 Pro e quer contribuir com dados:

```bash
# Compile e execute o diagnóstico
cargo build --release
./target/release/openrapoo-diag generate-report
```

Abra uma issue no GitHub e anexe o relatório gerado.

