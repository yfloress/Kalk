<div align="center">

<img src="./packaging/linux/kalk.svg" alt="Kalk" width="120" height="120" />

<h1>KALK</h1>

**Tu dashboard académico en la terminal.**

[![English](https://img.shields.io/badge/README-English-blue?style=flat-square)](README.md)
[![Rust](https://img.shields.io/badge/Hecho_con-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Licencia](https://img.shields.io/badge/Licencia-AGPL_v3-blue?style=flat-square)](LICENSE)

</div>

![Kalk](assets/screenshot.png)

---

## Qué es

Kalk responde la pregunta que una planilla nunca termina de contestar:
**¿voy a aprobar, y qué tengo que hacer al respecto?**

Le describes cómo se evalúa un ramo — sus categorías, sus ponderaciones, las
reglas enterradas en el programa — y Kalk te va diciendo dónde estás parado. No
es una estimación: lo que no tiene nota cuenta como cero, así que el número que
ves es el piso, y al lado tienes el techo al que todavía puedes llegar.

Corre en la terminal, todo con el teclado, y guarda tus datos como JSON plano
en tu propia máquina.

## Qué hace

**Sigue un ramo como se evalúa de verdad.** Categorías con peso, evaluaciones
dentro de ellas, y las reglas que traen los programas reales: descartar las N
peores, medias geométricas, promedios mínimos, mínimos por evaluación, topes de
nota, exigencias de asistencia y exámenes globales.

**Te dice qué necesitas.** Para cualquier evaluación sin nota calcula la marca
exacta que te haría pasar — y te avisa cuando ninguna nota alcanza, porque
alguna regla ya dejó la aprobación fuera de alcance.

**Conserva tus semestres.** Los anteriores se quedan, con un dashboard encima:
promedio y techo alcanzable, cuánto de la nota sigue en juego, créditos en
riesgo, y qué ramo necesita atención primero.

**Lee un programa por ti.** El asistente de importación te entrega un prompt
para pegar en cualquier IA junto al PDF del programa; pegas la respuesta de
vuelta y revisas cada regla antes de aplicarla.

**Habla español e inglés**, completos, intercambiables en cualquier momento.

## Sobre las notas

Las notas van de 0 a 100, se aprueba con 55 por defecto y es configurable por
ramo. El redondeo sigue la regla habitual: 54.5 pasa a 55, y aprueba.

Las evaluaciones sin nota siempre cuentan como cero. Es a propósito: hace que
la nota mostrada sea tu situación real y no un pronóstico optimista.

## Instalación

```bash
git clone https://github.com/yfloress/Kalk.git
cd Kalk
./install.sh --user      # en ~/.local
```

O `sudo ./install.sh` para instalarlo en el sistema. El script compila Kalk si
hace falta y deja en su lugar el binario, la entrada de escritorio y el icono;
con `--uninstall` los quita.

Necesitas [Rust](https://rustup.rs/) 1.85+, y una
[Nerd Font](https://www.nerdfonts.com/) si quieres los iconos — Kalk te
pregunta en el primer arranque y usa Unicode simple si dices que no.

¿Vas a trabajar en Kalk en vez de instalarlo? `nix run` lo levanta directo
desde el código. Para las notas de macOS, Fedora, Debian y Arch, mira la
[guía de instalación](docs/INSTALL_ES.md).

## Cómo moverse

Presiona `?` en cualquier momento para ver el mapa completo de teclas. Todo se
alcanza sin tocar el mouse.

Tus datos viven en `XDG_DATA_HOME/kalk` como JSON legible — respáldalos,
sincronízalos o edítalos a mano si quieres.

## Contribuir

[`AGENTS.md`](AGENTS.md) documenta la arquitectura y las convenciones que el
código se exige a sí mismo. `nix develop` te deja el entorno completo.

## Licencia

**GNU Affero General Public License v3.0 o posterior.** Ver [LICENSE](LICENSE).

<div align="center">

Hecho con 🦀 Rust y ❄️ Nix

</div>
