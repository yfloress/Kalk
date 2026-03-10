<div align="center">

# KALK

**Tu dashboard académico en la terminal.**

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

- **Escala**: 0-100 puntos
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

> Para instrucciones detalladas por plataforma (macOS, Fedora, Debian/Ubuntu, Arch Linux), consulta la [Guía de Instalación](docs/INSTALL.md).

### Requisitos

- [Rust & Cargo](https://rustup.rs/) (>= 1.85) o [Nix](https://nixos.org/)
- Se recomienda una [Nerd Font](https://www.nerdfonts.com/) (iconos activados por defecto — se pueden desactivar en Ajustes)

### Usando Cargo

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
cargo run --release
```

### Usando Nix

```bash
nix develop
cargo run
```

El entorno Nix incluye `cargo-audit` para análisis de seguridad.

---

## Atajos de Teclado

### Navegación

| Tecla | Acción |
|:-----:|--------|
| `Tab` | Ciclar foco entre paneles |
| `h` / `←` | Enfocar panel izquierdo |
| `l` / `→` | Enfocar panel derecho |
| `j` / `↓` | Moverse hacia abajo en la lista |
| `k` / `↑` | Moverse hacia arriba en la lista |

### Acciones

| Tecla | Acción |
|:-----:|--------|
| `n` | Crear nuevo item (ramo/categoría/evaluación) |
| `Enter` | Editar item seleccionado |
| `d` | Eliminar item seleccionado |
| `t` | Guardar ramo actual como plantilla |
| `b` | Auto-balancear pesos de categorías |
| `y` | Copiar (yank) la evaluación seleccionada |
| `p` | Pegar evaluación copiada en la categoría actual |
| `Ctrl+N` | Agregar múltiples evaluaciones en lote |
| `g` | Ingresar nota de examen global |
| `c` | Alternar vista compacta de ramos |
| `S` | Abrir ajustes |
| `L` | Cambiar idioma |
| `q` | Salir |

### En Popups de Edición

| Tecla | Acción |
|:-----:|--------|
| `Tab` | Cambiar entre campos de entrada |
| `Enter` | Confirmar |
| `Esc` | Cancelar |

### En Editor de Categoría

| Tecla | Acción |
|:-----:|--------|
| `Ctrl+R` | Mostrar/ocultar sección de reglas avanzadas |
| `?` | Mostrar/ocultar ayuda de campos |
| `Space` / `Enter` | Ciclar campos toggle (eliminar peores, método prom., etc.) |

### En Ajustes

| Tecla | Acción |
|:-----:|--------|
| `Space` | Alternar ajuste seleccionado |
| `Enter` | Confirmar y guardar |
| `Esc` | Cancelar sin guardar |

---

## Estructura del Proyecto

```
src/
├── main.rs          # Punto de entrada, configuración del terminal, panic hooks
├── app/
│   ├── mod.rs       # Struct App, estado, navegación, getters
│   ├── forms.rs     # Manejo de formularios: ramo/categoría/eval, eliminación, gestión de pesos
│   └── actions.rs   # Acciones secundarias: plantillas, idioma, ajustes, global, copiar/pegar, lote
├── model/
│   ├── mod.rs       # Evaluation, Course, NeededGrade, WeightValidation, plantillas
│   ├── category.rs  # Category, CategoryRules, AveragingMethod, MinimumNotMetAction
│   ├── global.rs    # Lógica de cálculo de nota con examen global
│   └── tests.rs     # Todos los tests unitarios del modelo
├── ui/
│   ├── mod.rs       # Layout principal, panel de ramos, panel de categorías
│   ├── panels.rs    # Panel de evaluaciones, renderizado del footer
│   ├── popups.rs    # Popups (plantilla, ramo, categoría, eliminar, idioma, guardar plantilla)
│   ├── eval_popups.rs # Popup de evaluación, nota global, agregar en lote
│   ├── settings_popup.rs # Popup de ajustes
│   ├── helpers.rs   # Helpers de renderizado compartidos y funciones de formato
│   ├── icons.rs     # Sets de iconos Nerd Font y fallback Unicode
│   └── theme.rs     # Tema de colores semántico (bordes redondeados, paleta)
├── templates.rs     # Plantillas predefinidas (según idioma)
├── events.rs        # Manejo de eventos de teclado y despacho
├── i18n.rs          # Traducciones (inglés + español)
└── persistence.rs   # Almacenamiento JSON (dirs XDG, escritura atómica)

~/.local/share/kalk/
├── data.json           # Tus datos de ramos
├── config.json         # Tu configuración (idioma, nerd fonts)
└── user_templates.json # Tus plantillas personalizadas
```

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