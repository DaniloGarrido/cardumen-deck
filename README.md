<p align="center">
  <img src="docs/assets/octopus.webp" alt="Cardumen Deck" width="112" />
</p>

<h1 align="center">Cardumen Deck</h1>

<p align="center">
  <strong>Many services, one current.</strong><br />
  Muchos servicios, una corriente.
</p>

<p align="center">
  <a href="https://github.com/DaniloGarrido/cardumen-deck/actions/workflows/ci.yml"><img src="https://github.com/DaniloGarrido/cardumen-deck/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/DaniloGarrido/cardumen-deck/releases"><img src="https://img.shields.io/github/v/release/DaniloGarrido/cardumen-deck?display_name=tag&sort=semver" alt="Última versión" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/DaniloGarrido/cardumen-deck" alt="Licencia MIT" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4" alt="Windows 10/11" />
  <img src="https://img.shields.io/badge/telemetry-none-success" alt="Sin telemetría" />
</p>

<p align="center">
  <a href="README.en.md">English</a> ·
  <a href="#descarga">Descarga</a> ·
  <a href="PRIVACY.md">Privacidad</a> ·
  <a href="SECURITY.md">Seguridad</a> ·
  <a href="AI.md">IA</a> ·
  <a href="LEGAL.md">Legal</a> ·
  <a href="CHANGELOG.md">Registro de cambios</a>
</p>

Panel local para Windows que centraliza el arranque, monitoreo y operación de servicios de desarrollo.

> **Estado:** `0.1.x` en evolución, usable para desarrollo diario. Solo Windows por ahora.

> **Uso previsto:** herramienta local para desarrollo. No es un servicio gestionado ni un componente de producción. Puedes usarla en tu flujo real de desarrollo; eres responsable de los comandos, credenciales, datos y copias de seguridad.

![Panel de Cardumen Deck](docs/screenshots/dashboard.webp)

## Características

- Espacios de trabajo para agrupar servicios.
- Inicio, detención y reinicio de procesos locales.
- Detección de servicios activos y reconciliación de procesos huérfanos.
- Métricas de CPU, RAM y cantidad de procesos.
- Verificaciones de estado HTTP, TCP y por proceso.
- Registros por ejecución y visor integrado.
- Integración Git: estado, ramas, checkout, pull, push, merge y conflictos.
- Detección de dependencias y entorno.
- Formulario guiado para crear servicios.
- Temas claro/oscuro y varias paletas.
- Registro interno de errores y actividad.

![Configuración de Cardumen Deck](docs/screenshots/settings.webp)

## Privacidad y seguridad

- **Sin telemetría:** no hay analíticas, reportes remotos de errores, identificadores de usuario ni backend propio.
- **Red limitada:** solo verificaciones de estado HTTP/TCP, operaciones Git y apertura de URLs configuradas.
- **Datos locales:** configuración, estado y registros se guardan en el equipo. No hay sincronización en la nube.
- **Sin cifrado en reposo:** la configuración y los registros no están cifrados; pueden contener secretos.
- **Sin entorno aislado (sandbox):** los comandos se ejecutan con los privilegios de tu usuario de Windows.

Lee [PRIVACY.md](PRIVACY.md) y [SECURITY.md](SECURITY.md) para el detalle completo.

## Descarga

La forma recomendada es usar la última versión publicada:

- [Descargar la última versión](https://github.com/DaniloGarrido/cardumen-deck/releases)
- **Versión portable:** `CardumenDeck-v*-windows-x64-portable.exe`. No requiere instalación.
- **Instalador:** `CardumenDeck-v*-windows-x64-setup.exe`. Instala la aplicación con NSIS.
- Cada versión incluye `SHA256SUMS.txt` para verificar la integridad.

> La versión `0.1.x` puede estar sin firmar. Windows SmartScreen puede mostrar una advertencia. Verifica el SHA256 antes de ejecutar y no desactives tu antivirus.

### Verificar el SHA256

```powershell
Get-FileHash .\CardumenDeck-v0.1.0-windows-x64-portable.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

## Requisitos

### Para usar la aplicación

- Windows 10 u 11.
- WebView2 Runtime. Windows 10/11 modernos ya lo incluyen.
- Git para Windows si vas a usar las funciones Git.
- Git Credential Manager o credenciales configuradas si vas a hacer `push`/`pull` remotos.

No necesitas Node.js ni Rust para usar el portable o el instalador.

### Para desarrollar

- Node.js 24+ y npm 10+.
- Rust stable 1.85+.
- Visual Studio Build Tools con C++.
- WebView2 Runtime.
- Git para Windows.

## Primeros pasos

1. Abre Cardumen Deck.
2. Crea un espacio de trabajo.
3. Agrega un servicio con nombre, carpeta de trabajo y comando.
4. Inicia el servicio desde su tarjeta.
5. Configura una verificación de estado HTTP, TCP o por proceso si quieres ver su estado.

## Datos y privacidad

- Configuración: `%APPDATA%\CardumenDeck\config.json`
- Estado de ejecución: `%APPDATA%\CardumenDeck\runtime.json`
- Registros (logs): `%LOCALAPPDATA%\CardumenDeck\logs\`

Cardumen Deck no tiene backend remoto, analíticas ni telemetría. Las verificaciones de estado consultan únicamente las URLs o puertos que configures. Las operaciones Git contactan el remoto solo cuando las ejecutas.

Las variables de entorno se guardan en `config.json` sin cifrar. Los registros pueden contener comandos, rutas, salida de procesos y secretos impresos por un servicio. Revisa y protege esos archivos antes de compartirlos.

Windows, WebView2, Git y los puntos de conexión configurados pueden tener sus propias políticas y conexiones. Consulta [PRIVACY.md](PRIVACY.md) para la tabla completa de datos y red.

Si actualizas desde una versión anterior, la configuración, el estado de ejecución y el registro del panel se migran automáticamente la primera vez.

## Desinstalar

- **Versión portable:** elimina el `.exe`.
- **Instalador:** usa “Agregar o quitar programas” o el desinstalador de NSIS.
- Para borrar también tus datos, elimina:
  - `%APPDATA%\CardumenDeck`
  - `%LOCALAPPDATA%\CardumenDeck`

Después de migrar, las carpetas antiguas de `ServiceDashboard` se pueden eliminar si ya no las necesitas.

## Limitaciones conocidas

- Solo Windows.
- No hay actualización automática; descarga una nueva versión manualmente.
- La versión `0.1.x` puede estar sin firma digital.
- Las funciones Git requieren Git instalado y credenciales configuradas para operaciones remotas.
- El instalador NSIS y la versión portable comparten la misma configuración local.
- No hay entorno aislado (sandbox): los comandos se ejecutan con los privilegios del usuario.
- La configuración y los registros no están cifrados.
- No hay auditoría independiente, certificación, acuerdo de nivel de servicio (SLA) ni programa de recompensas por errores.
- No hay gestión central ni telemetría para administrar varias instalaciones.

## Solución de problemas

### SmartScreen bloquea el ejecutable

Es esperado en binarios sin firma. Verifica el SHA256 y usa “Más información” → “Ejecutar de todas formas” solo si confías en el archivo.

### La aplicación no abre o se ve en blanco

Verifica que WebView2 Runtime esté instalado y actualizado. También revisa los registros en `%LOCALAPPDATA%\CardumenDeck\logs\`.

### Git no se detecta

Instala Git for Windows y asegúrate de que `git` esté en `PATH`. Reinicia Cardumen Deck después.

### Un servicio no arranca

Revisa la carpeta de trabajo, el comando, los argumentos y el log de la ejecución desde la tarjeta del servicio. Si el comando funciona en una terminal pero no en la aplicación, revisa diferencias de `PATH` y variables de entorno.

### El puerto 1420 está ocupado en desarrollo

Cierra el proceso que esté usando el puerto o reinicia la sesión de desarrollo.

## Tecnologías

- **Interfaz:** Vue 3, TypeScript, Vite, Lucide.
- **Backend:** Rust, Tauri 2, Tokio.
- **Motor de interfaz:** WebView2.
- **Datos:** archivos JSON locales, sin backend remoto.

## Desarrollo

```powershell
git clone https://github.com/DaniloGarrido/cardumen-deck.git
cd cardumen-deck
npm install
npm run dev
```

La aplicación de escritorio se abre con la interfaz en modo de desarrollo.

## Compilación

Desde la raíz del repositorio:

```powershell
# Versión portable
npm run build -- --no-bundle

# Versión portable + instalador NSIS
npm run build
```

Rutas de salida:

- Versión portable: `Backend/target/release/cardumen-deck.exe`
- Instalador: `Backend/target/release/bundle/nsis/`

## Verificación

```powershell
npm run lint
npm run check
npm run build
cargo fmt --manifest-path Backend/Cargo.toml -- --check
cargo clippy --manifest-path Backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path Backend/Cargo.toml --lib
```

CI ejecuta lint, verificación de tipos, compilación de la interfaz, formato, Clippy y pruebas del backend en Windows.

## Versiones y firma

El flujo de trabajo de publicación está configurado para construir la versión portable, el instalador y `SHA256SUMS.txt` a partir de una etiqueta `v*`.

La firma de código con **SignPath Foundation** está pendiente de aprobación. Mientras tanto, el editor visible puede ser desconocido y SmartScreen puede advertir. Ver [CODE_SIGNING.md](CODE_SIGNING.md).

## Seguridad

Cardumen Deck aplica una Política de seguridad de contenido (CSP) restrictiva, capacidades (capabilities) acotadas y no tiene backend remoto ni telemetría. No hay entorno aislado (sandbox), cifrado en reposo ni auditoría independiente. Para reportar vulnerabilidades, usa GitHub Security Advisories. No abras una incidencia pública con detalles explotables. Ver [SECURITY.md](SECURITY.md).

## IA en el desarrollo

Parte del código y la documentación fue desarrollada con asistencia de inteligencia artificial. La aplicación no usa inteligencia artificial en tiempo de ejecución ni envía instrucciones a APIs de inteligencia artificial. Los mantenedores revisan y prueban el código, pero la inteligencia artificial puede introducir errores. Ver [AI.md](AI.md).

## Contribuir

Lee [CONTRIBUTING.md](CONTRIBUTING.md) y [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Las contribuciones asistidas por inteligencia artificial deben cumplir la política de [AI.md](AI.md).

## Soporte

El soporte es comunitario y de mejor esfuerzo. No hay acuerdo de nivel de servicio (SLA), soporte comercial ni programa de recompensas por errores. Consulta [SUPPORT.md](SUPPORT.md).

## Aviso legal

Cardumen Deck se distribuye bajo la licencia MIT. El software se proporciona “tal cual”, sin garantía, y el uso es responsabilidad del usuario. No hay afiliación con Microsoft, Tauri, Vue, Rust, Lucide o GitHub. Consulta [LEGAL.md](LEGAL.md).

## Estructura

```text
.
├── Backend/            # Tauri + Rust
├── FrontEnd/           # Vue 3 + TypeScript + Vite
├── docs/               # Arquitectura, capturas y recursos
├── .github/            # CI, versiones y plantillas
├── package.json
├── README.md
└── README.en.md
```

## Documentación técnica

- [Arquitectura](docs/ARCHITECTURE.md)
- [Privacidad](PRIVACY.md)
- [Seguridad](SECURITY.md)
- [IA en el desarrollo](AI.md)
- [Aviso legal](LEGAL.md)
- [Avisos de terceros](THIRD_PARTY_NOTICES.md)
- [Soporte](SUPPORT.md)
- [Política de firma](CODE_SIGNING.md)
- [Registro de cambios](CHANGELOG.md)

## Créditos

- Mascota: pulpo de Cardumen Deck.
- Construido con [Tauri](https://tauri.app/), [Vue](https://vuejs.org/) y [Rust](https://www.rust-lang.org/).

## Licencia

MIT. Ver [LICENSE](LICENSE).
