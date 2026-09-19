# Guia de Solução de Problemas do OpenRapoo

Problemas frequentes, diagnóstico de permissões e procedimentos de solução.

---

## 🔍 Problemas de Permissão (`Permissão Negada` em `/dev/input/event*` ou `/dev/hidraw*`)

### Causa
Seu usuário não possui permissão de leitura/escrita nos nós de dispositivos de entrada do kernel.

### Solução
1. Verifique se seu usuário pertence ao grupo `input`:
   ```bash
   groups $USER
   ```
2. Se o grupo `input` não constar na lista, adicione seu usuário:
   ```bash
   sudo usermod -aG input $USER
   ```
3. Reinstale as regras UDev:
   ```bash
   sudo cp udev/99-openrapoo.rules /etc/udev/rules.d/
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```
4. Encerre a sessão e entre novamente no sistema.

---

## ⚡ Aviso de Conexão com o Daemon

### Causa
A interface `openrapoo-gui` não conseguiu conectar ao soquete Unix do `openrapoo-daemon` (`$XDG_RUNTIME_DIR/openrapoo.sock`).

### Solução
1. Verifique se o daemon está em execução:
   ```bash
   pgrep -fl openrapoo-daemon
   ```
2. Inicie o daemon manualmente para inspecionar mensagens de log:
   ```bash
   openrapoo-daemon
   ```

---

## 🔋 Telemetria de Bateria Exibe `Indisponível` ou `Leitura Antiga`

### Causa
- Modo Cabo USB: O transporte por cabo alimenta o mouse diretamente pela porta USB e não transmite relatórios sem fio.
- Modo Bluetooth: A telemetria é fornecida via BlueZ D-Bus (`org.bluez.Battery1`). Se o BlueZ não consultou o serviço GATT recentemente, o status é marcado como `Leitura Antiga`.

### Solução
Pressione **Atualizar Bateria Agora** na aba de Diagnóstico de Bateria para solicitar uma varredura ao daemon.

