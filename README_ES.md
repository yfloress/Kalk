<div align="center">

<img src="./packaging/linux/kalk.svg" alt="Kalk" width="120" height="120" />

<h1>KALK</h1>

**Tu dashboard académico en la terminal.**

[![English](https://img.shields.io/badge/README-English-blue?style=flat-square)](README.md)

Gestiona ramos, registra notas por categorías y calcula automáticamente la nota que necesitas para aprobar — todo sin tocar el mouse.

[![Rust](https://img.shields.io/badge/Hecho_con-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![TUI](https://img.shields.io/badge/Interfaz-Ratatui-green?style=for-the-badge)](https://github.com/ratatui-org/ratatui)
[![License](https://img.shields.io/badge/Licencia-AGPL_v3-blue?style=for-the-badge)](LICENSE)

</div>

![IMG](.img/img.png)

---

## Características

| Característica | Descripción |
|----------------|-------------|
| **Semestres** | Agrupa ramos por semestre y conserva los anteriores — no borres un ramo para empezar de nuevo |
| **Dashboard** | Pantalla Home con promedio del semestre, aprobados/reprobados, créditos en riesgo y el ramo más crítico |
| **Créditos** | Créditos opcionales por ramo (SCT, ECTS, ...) — los promedios se ponderan cuando están |
| **Sistema jerárquico de notas** | Estructura Ramo → Categorías → Evaluaciones con ponderaciones |
| **Evaluaciones ponderadas** | Asigna pesos individuales a las evaluaciones dentro de una categoría (ej. 20%/40%/40%) |
| **Cálculo en tiempo real** | Muestra exactamente qué nota necesitas en cada evaluación para aprobar |
| **Examen global** | Configura políticas de examen global: promedio ponderado con semestre, o reemplazar peor categoría |
| **Reglas avanzadas por categoría** | Eliminar peores notas, media geométrica, promedios mínimos, mínimo por evaluación, evaluaciones ponderadas, redondeo por categoría |
| **Plantillas de ramos** | Plantillas predefinidas + crea tus propias plantillas reutilizables con `t` |
| **Validación de pesos** | Indicadores visuales cuando los pesos no suman 100% |
| **Auto-balance** | Distribuye automáticamente los pesos equitativamente entre categorías |
| **Copiar y Pegar** | Copia evaluaciones entre categorías con `y`/`p` |
| **Agregar en lote** | Agrega múltiples evaluaciones de una vez con `Ctrl+N` |
| **Vista compacta** | Alterna la vista compacta del panel de ramos con `c` para más espacio |
| **Iconos Nerd Font** | Iconos elegantes con Nerd Fonts (activados por defecto), con fallback Unicode |
| **UI con temas** | Bordes redondeados, colores semánticos y atajos estilizados |
| **Ajustes** | Activar/desactivar iconos Nerd Font, cambiar idioma — todo persistido en disco |
| **Bilingüe** | Soporte completo en inglés y español (`L` para cambiar) |
| **Persistencia automática** | Los datos se guardan localmente (`XDG_DATA_HOME/kalk`) y persisten entre sesiones |
| **Solo teclado** | TUI rápido y ligero — no se necesita mouse |

---

## Cálculo de Notas

- **Escala**: 0-100 puntos (la única admitida por ahora)
- **Nota de aprobación por defecto**: 55 (configurable por ramo)
- **Redondeo**: 0.5+ redondea hacia arriba (entonces 54.5 → 55 = aprobado)
- **Indicadores por evaluación**: Muestra "Necesitas X+ en esta eval para aprobar" al editar

---

## Reglas Avanzadas por Categoría

Cada categoría soporta reglas avanzadas opcionales (`Shift+A` al editar):

| Regla | Descripción |
|-------|-------------|
| **Eliminar Peores** | Descarta las N peores notas antes de promediar (0–5) |
| **Método de Promedio** | Aritmético (por defecto) o Media Geométrica |
| **Evaluaciones Ponderadas** | Asigna pesos porcentuales individuales a las evaluaciones en vez de promediar equitativamente |
| **Promedio Mínimo** | Exigir un promedio mínimo en esta categoría para aprobar |
| **Min. Por Eval** | Nota mínima requerida en cada evaluación individual |
| **Min. Una Eval** | Al menos una evaluación debe alcanzar una nota mínima |
| **Si No Se Cumple** | Cuando no se alcanza el mínimo: final = prom. categoría, requiere global, o reprueba ramo |
| **Redondear Categoría** | Redondea el promedio de la categoría antes de ponderar |

Presiona `?` dentro del editor de categoría para ver la ayuda completa.

---

## Instalación

> Para instrucciones detalladas por plataforma (macOS, Fedora, Debian/Ubuntu, Arch Linux), consulta la [Guía de Instalación](docs/INSTALL_ES.md).

### Requisitos

- [Rust & Cargo](https://rustup.rs/) (>= 1.85) o [Nix](https://nixos.org/)
- Se recomienda una [Nerd Font](https://www.nerdfonts.com/) (iconos activados por defecto — se pueden desactivar en Ajustes)

### Usando Cargo

```bash
git clone https://github.com/yfloress/Kalk.git
cd Kalk
cargo run --release
```

### Instalacion Rapida (Linux)

```bash
sudo ./install.sh        # todo el sistema
./install.sh --user      # solo tu usuario (~/.local)
```

Instala el binario, acceso directo de escritorio e icono (solo el binario en macOS). Ver [El script de instalación](docs/INSTALL_ES.md#el-script-de-instalación).

### Usando Nix

```bash
nix develop
cargo run
```

El entorno Nix incluye `cargo-audit` para análisis de seguridad.

---
 
## Atajos de Teclado
 
Presiona `?` dentro de la app para ver la referencia completa.
 
---

## Stack Tecnológico

| Herramienta | Propósito |
|-------------|-----------|
| [Ratatui](https://github.com/ratatui-org/ratatui) | Renderizado TUI |
| [Crossterm](https://github.com/crossterm-rs/crossterm) | Backend de terminal |
| [Serde](https://serde.rs/) | Serialización JSON |
| [color-eyre](https://github.com/yaahc/color-eyre) | Manejo de errores |

---

## Desarrollo

```bash
# Entrar al entorno de desarrollo
nix develop

# Ejecutar tests
cargo test

# Lint
cargo clippy

# Auditoría de seguridad
cargo audit
```

---

## Licencia

**GNU Affero General Public License v3.0 (AGPL-3.0)**

Ver [LICENSE](LICENSE) para más detalles.

---

<div align="center">

Hecho con ❤️, 🦀 Rust y ❄️ Nix

</div>
