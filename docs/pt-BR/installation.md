# Guia de Instalação e Implantação do OpenRapoo

Este documento detalha todos os métodos de instalação disponíveis para o OpenRapoo, incluindo pacotes Debian `.deb`, `AppImage` portátil, compilação via código fonte e ativação de serviços de usuário.

---

## 📦 Opções de Instalação

### 1. Pacote Debian / Ubuntu (.deb)

Método recomendado para distribuições baseadas em Debian (Ubuntu 22.04 / 24.04, Debian 12, Linux Mint, Pop!_OS):

```bash
# Baixar a release mais recente
wget https://github.com/openrapoo/openrapoo/releases/download/v0.1.0/openrapoo_0.1.0_amd64.deb

# Instalar o pacote
sudo dpkg -i openrapoo_0.1.0_amd64.deb

# Resolver dependências ausentes se necessário
sudo apt install -f
```

O pacote instala automaticamente os binários em `/usr/bin/`, os arquivos de atalho em `/usr/share/applications/`, as regras UDev em `/lib/udev/rules.d/99-openrapoo.rules` e o serviço de usuário systemd em `/usr/lib/systemd/user/openrapoo-daemon.service`.

### 2. AppImage Standalone

Compatível com qualquer distribuição Linux:

```bash
# Baixar o AppImage
wget https://github.com/openrapoo/openrapoo/releases/download/v0.1.0/OpenRapoo-0.1.0-x86_64.AppImage

# Conceder permissão de execução
chmod +x OpenRapoo-0.1.0-x86_64.AppImage

# Executar a aplicação
./OpenRapoo-0.1.0-x86_64.AppImage
```

---

## 🔑 Configuração de Permissões UDev

Para garantir que o OpenRapoo possa detectar e controlar o hardware sem `sudo`:

```bash
# Adicionar o usuário ao grupo input
sudo usermod -aG input $USER

# Recarregar as regras udev
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Encerre a sessão e entre novamente para que as alterações de grupo entrem em vigor.

