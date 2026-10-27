# Avisos de terceros

> Versión en inglés: [THIRD_PARTY_NOTICES.en.md](THIRD_PARTY_NOTICES.en.md). La versión en español es la principal. En caso de conflicto, prevalece esta versión. [LICENSE](LICENSE) prevalece sobre este documento.

## Alcance

Cardumen Deck usa componentes de terceros. Cada componente conserva su propia licencia y sus propios términos. Este documento es un resumen y no reemplaza las licencias originales.

## Componentes principales

| Componente | Uso | Licencia o términos | Enlace |
|---|---|---|---|
| Tauri | Marco de trabajo de escritorio | MIT o Apache-2.0 | https://github.com/tauri-apps/tauri |
| Vue | Interfaz | MIT | https://github.com/vuejs/core |
| Rust | Lenguaje y cadena de herramientas | MIT o Apache-2.0 | https://www.rust-lang.org/ |
| Lucide | Iconos | ISC | https://github.com/lucide-icons/lucide |
| WebView2 | Motor de interfaz en Windows | Términos de Microsoft | https://developer.microsoft.com/microsoft-edge/webview2/ |
| NSIS | Instalador de Windows | Licencia de NSIS | https://nsis.sourceforge.io/ |

## Dependencias transitivas

Las dependencias exactas y sus versiones están registradas en:

- `Backend/Cargo.lock` para Rust.
- `package-lock.json` para npm.
- `Backend/Cargo.toml` y `package.json` para dependencias directas.

Las licencias exactas deben verificarse en los proyectos originales y en los archivos de licencia incluidos por cada dependencia.

## Distribución de binarios

Si distribuyes Cardumen Deck o una versión modificada, debes conservar los avisos de copyright y las licencias requeridas por los componentes de terceros. Este documento no sustituye esos avisos.

## Marcas

Las marcas, nombres comerciales y logotipos pertenecen a sus respectivos titulares. Cardumen Deck no está afiliado ni respaldado por esos proyectos.

## Sin asesoría legal

Este documento no constituye asesoría legal. Para una revisión de licencias o distribución, consulta a un profesional habilitado.
