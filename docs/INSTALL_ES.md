# Guía de Instalación

Kalk es una aplicación de terminal escrita íntegramente en Rust. **No tiene
dependencias de C** — no necesitas `gcc`, `pkg-config` ni paquetes `-dev` /
`-devel`, solo el toolchain de Rust. Usa la **edición 2024** de Rust, que
requiere **Rust 1.85 o superior**.

Plataformas soportadas: **macOS** y **Linux** (Fedora, Debian/Ubuntu, Arch y
cualquier otra distribución con un toolchain de Rust reciente).

> **Resumen rápido** — instala Rust y luego:
> ```bash
> git clone https://codeberg.org/Kyronix/Kalk.git
> cd Kalk
> sudo ./install.sh        # o bien: ./install.sh --user  (sin sudo)
> ```

---

## Tabla de Contenidos

- [Requisitos](#requisitos)
- [Paso 1 — Instalar Rust y Git](#paso-1--instalar-rust-y-git)
- [Paso 2 — Obtener el código fuente](#paso-2--obtener-el-código-fuente)
- [Paso 3 — Compilar e instalar](#paso-3--compilar-e-instalar)
- [El script de instalación](#el-script-de-instalación)
- [Nerd Font (opcional)](#nerd-font-opcional)
- [Ejecutar Kalk](#ejecutar-kalk)
- [Ubicación de los datos](#ubicación-de-los-datos)
- [Desinstalar](#desinstalar)

---

## Requisitos

| Requisito | Detalles |
|-----------|----------|
| **Rust** | >= 1.85 (edición 2024) |
| **Git** | Para clonar el repositorio |
| **Nerd Font** | Opcional — para los iconos (se puede desactivar en Ajustes) |

---

## Paso 1 — Instalar Rust y Git

Elige tu plataforma. Después, verifica el toolchain en cualquier plataforma con:

```bash
rustc --version   # debe indicar 1.85 o superior
```

### macOS

Instala Rust con [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Git viene con las Xcode Command Line Tools:

```bash
xcode-select --install
```

### Fedora

```bash
sudo dnf install git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

El paquete del sistema (`sudo dnf install rust cargo`) también sirve **si**
provee Rust >= 1.85; si no, usa rustup como se muestra arriba.

### Debian / Ubuntu

El Rust de los repositorios de Debian/Ubuntu suele ser muy antiguo, así que usa
[rustup](https://rustup.rs/):

```bash
sudo apt update && sudo apt install git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Arch Linux

Arch trae un Rust actualizado:

```bash
sudo pacman -S rust git
```

O gestiona toolchains con rustup: `sudo pacman -S rustup && rustup default stable`.

---

## Paso 2 — Obtener el código fuente

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
```

---

## Paso 3 — Compilar e instalar

El script `install.sh` provisto es la vía recomendada — **compila el binario
de release por ti** (ejecuta `cargo build --release` solo cuando el binario no
existe o cambiaron las fuentes) y luego lo instala.

```bash
sudo ./install.sh          # instalación para todo el sistema en /usr/local
./install.sh --user        # instalación solo para tu usuario en ~/.local (sin sudo)
```

Lo que se instala depende de la plataforma:

| Plataforma | Instala |
|------------|---------|
| **Linux** | Binario **+** acceso directo de escritorio **+** icono de la aplicación |
| **macOS** | **Solo** el binario (los accesos directos e iconos son exclusivos de Linux y se omiten automáticamente) |

### ¿Cuál usar en macOS?

- **`sudo ./install.sh`** instala en `/usr/local/bin/kalk`, que ya está en el
  `PATH` por defecto de macOS, así que `kalk` funciona de inmediato. Es la
  opción más simple. (Los usuarios administradores de macOS tienen `sudo`.)
- **`./install.sh --user`** instala en `~/.local/bin/kalk` sin `sudo`, pero
  `~/.local/bin` **no** está en el `PATH` por defecto de macOS. Agrégalo una vez:
  ```bash
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
  ```

En Linux, `--user` suele ser lo más cómodo — `~/.local/bin` normalmente ya está
en el `PATH`, así que no hace falta `sudo` ni configuración adicional.

### Instalación manual (cualquier plataforma)

Si prefieres no usar el script, compila y copia el binario tú mismo:

```bash
cargo build --release
sudo cp target/release/kalk /usr/local/bin/
```

---

## El script de instalación

```bash
sudo ./install.sh                 # todo el sistema   (PREFIX=/usr/local)
./install.sh --user               # solo tu usuario   (PREFIX=~/.local)
sudo ./install.sh --uninstall     # eliminar una instalación de sistema
./install.sh --user --uninstall   # eliminar una instalación de usuario
PREFIX=/usr sudo ./install.sh     # prefijo personalizado
```

El script:

- Se niega a ejecutarse en plataformas no soportadas (cualquier cosa que no sea Linux/macOS).
- En macOS, avisa que solo se instala el binario.
- Compila el binario con `cargo build --release --locked` si hace falta.
- Nunca toca tu directorio de datos — ver [Desinstalar](#desinstalar).

Artefactos instalados en **Linux**:

| Artefacto | Ruta de sistema (`sudo`) | Ruta de usuario (`--user`) |
|-----------|--------------------------|----------------------------|
| Binario | `/usr/local/bin/kalk` | `~/.local/bin/kalk` |
| Acceso directo | `/usr/local/share/applications/kalk.desktop` | `~/.local/share/applications/kalk.desktop` |
| Icono | `/usr/local/share/pixmaps/kalk.png` | `~/.local/share/pixmaps/kalk.png` |

El acceso directo usa `Terminal=true`, así que abrir Kalk desde el menú de
aplicaciones abre una terminal, ejecuta la app y la cierra al salir. El
instalador también refresca la caché de iconos y la base de datos de escritorio
para que la entrada aparezca sin volver a iniciar sesión.

**Icono personalizado:** reemplaza `packaging/linux/kalk.png` con tu propio PNG
(mismo nombre de archivo) antes de ejecutar el script.

---

## Nerd Font (opcional)

Kalk usa glifos de Nerd Font para los iconos. Se pueden desactivar en Ajustes,
pero una fuente con parche hace que la interfaz se vea mejor.

**macOS** — vía Homebrew (o instala cualquier fuente con parche desde Font Book):

```bash
brew install --cask font-jetbrains-mono-nerd-font
```

**Fedora / Debian / Ubuntu** — descarga una fuente con parche y refresca la caché:

```bash
mkdir -p ~/.local/share/fonts && cd ~/.local/share/fonts
curl -fLO https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.tar.xz
tar -xf JetBrainsMono.tar.xz
fc-cache -fv
```

**Arch Linux**:

```bash
sudo pacman -S ttf-jetbrains-mono-nerd
```

Luego configura tu emulador de terminal para usar la Nerd Font instalada.

---

## Ejecutar Kalk

Tras instalar:

```bash
kalk
```

O ejecútalo directamente desde el directorio del proyecto sin instalar:

```bash
cargo run --release
```

Presiona `?` dentro de cualquier popup de edición para ver la ayuda contextual,
o consulta el [README](../README_ES.md) para la referencia completa de atajos.

---

## Ubicación de los datos

Kalk guarda todo en el directorio de datos estándar de tu plataforma:

| Plataforma | Directorio |
|------------|------------|
| Linux | `~/.local/share/kalk/` |
| macOS | `~/Library/Application Support/kalk/` |

Ese directorio contiene:

| Archivo | Propósito |
|---------|-----------|
| `data.json` | Datos de los ramos |
| `config.json` | Ajustes (idioma, iconos) |
| `user_templates.json` | Plantillas de ramos personalizadas |

---

## Desinstalar

Usa el script para eliminar lo que instaló (el binario, y en Linux el acceso
directo y el icono):

```bash
sudo ./install.sh --uninstall      # todo el sistema
./install.sh --user --uninstall    # solo tu usuario
```

El script **nunca** elimina tu directorio de datos. Para borrarlo manualmente:

```bash
rm -rf ~/.local/share/kalk                       # Linux
rm -rf ~/Library/Application\ Support/kalk        # macOS
```
