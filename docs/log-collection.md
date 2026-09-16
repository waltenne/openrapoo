# Como Coletar Logs para o OpenRapoo

Este guia explica como coletar dados de diagnóstico do seu Rapoo MT760 Pro
e contribuir com eles ao projeto. Os logs não contêm informações pessoais.

---

## Método rápido (recomendado)

```bash
# Compile o openrapoo-diag
git clone https://github.com/openrapoo/openrapoo
cd openrapoo
cargo build --release

# Gere o relatório completo
./target/release/openrapoo-diag generate-report
```

O relatório é salvo em `~/.local/share/openrapoo/report-*.md`.  
Abra uma issue em https://github.com/openrapoo/openrapoo/issues e anexe o arquivo.

---

## Método manual (sem compilar)

Se você não tem Rust instalado, pode coletar os dados manualmente:

### 1. Identificar o dispositivo

```bash
# Listar todos os dispositivos USB (procure por Rapoo ou 24ae)
lsusb

# Exemplo de saída:
# Bus 001 Device 006: ID 24ae:2018 Shenzhen Rapoo Technology Co., Ltd.
```

### 2. Ver entradas de input

```bash
cat /proc/bus/input/devices | grep -A 15 -i rapoo
# ou
cat /proc/bus/input/devices | grep -A 15 "24ae"
```

### 3. Capturar eventos (precisa de sudo ou grupo input)

```bash
# Substitua eventX pelo número correto do seu mouse
sudo evtest /dev/input/eventX
# Pressione cada botão do mouse e veja os códigos
# Ctrl+C para parar
```

### 4. Ver HID descriptor

```bash
# Substitua pelo seu dispositivo HID real
# Formato: 0003:24AE:XXXX.YYYY
ls /sys/bus/hid/devices/ | grep 24AE

# Ler o descritor
hexdump -C /sys/bus/hid/devices/0003:24AE:XXXX.YYYY/report_descriptor
```

### 5. Ver logs do kernel

```bash
dmesg | grep -i rapoo
dmesg | grep -i "24ae"
dmesg | grep -i "hid-generic"
```

### 6. Informações do sistema

```bash
uname -r          # versão do kernel
cat /etc/os-release | grep PRETTY_NAME  # distro
```

---

## O que incluir na issue

Por favor, inclua:

1. Saída do `lsusb | grep 24ae`
2. Saída do `cat /proc/bus/input/devices` (trecho do Rapoo)
3. Saída do `evtest` ao pressionar cada botão
4. Saída do `hexdump` do report descriptor (se acessível)
5. Versão do kernel (`uname -r`)
6. Modo de conexão usado (USB-C, 2.4 GHz, Bluetooth)
7. Qualquer comportamento inesperado observado

---

## Privacidade

Os dados coletados contêm apenas:
- Identificadores do hardware (VID, PID, nome do dispositivo)
- Códigos de eventos de input (números inteiros)
- Informações do kernel e sistema operacional

**Não contêm:** nomes de usuário, endereços de email, conteúdo de arquivos, histórico de navegação ou qualquer dado pessoal.
