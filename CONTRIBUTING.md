# Contribuir

Gracias por querer contribuir a Cardumen Deck.

## Antes de empezar

- Busca incidencias existentes antes de abrir una nueva.
- Para cambios grandes, abre una incidencia primero para acordar el enfoque.
- No incluyas secretos, tokens, certificados ni rutas personales.
- No publiques datos de clientes, código confidencial ni información interna.

## Entorno

Requisitos:

- Windows 10/11.
- Node.js 24+ y npm 10+.
- Rust stable 1.85+.
- Visual Studio Build Tools con C++.
- WebView2 Runtime.
- Git for Windows.

## Desarrollo

```powershell
npm install
npm run dev
```

## Verificación

Antes de abrir una solicitud de extracción (pull request):

```powershell
npm run lint
npm --workspace FrontEnd run check
npm --workspace FrontEnd run build
cargo fmt --manifest-path Backend/Cargo.toml -- --check
cargo clippy --manifest-path Backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path Backend/Cargo.toml --lib
```

## Contribuciones asistidas por inteligencia artificial

Si usas herramientas de inteligencia artificial:

- revisa el resultado antes de enviarlo;
- verifica que las licencias y la procedencia sean compatibles;
- no incluyas código propietario, confidencial o sin permiso;
- agrega o actualiza pruebas cuando corresponda;
- describe el cambio y su impacto;
- no pegues código confidencial en herramientas no aprobadas.

Los mantenedores pueden rechazar contribuciones que no cumplan estos requisitos. Consulta [AI.md](AI.md).

## Seguridad

- No incluyas secretos en incidencias, registros, capturas ni solicitudes de extracción.
- Para vulnerabilidades, usa GitHub Security Advisories. No abras una incidencia pública con detalles explotables.
- Si el cambio afecta permisos, red, ejecución de comandos, Git o manejo de archivos, describe el impacto de seguridad.
- Consulta [SECURITY.md](SECURITY.md).

## Solicitudes de extracción

- Mantén el cambio enfocado.
- Explica qué problema resuelve.
- Incluye pasos para probarlo.
- Adjunta una captura o un video si cambia la interfaz.
- Actualiza la documentación si cambia el comportamiento.
- Indica si el cambio fue asistido por inteligencia artificial.

## Confirmaciones (commits)

Usa mensajes claros y en imperativo. Ejemplos:

- `feat: add service health filters`
- `fix: keep service actions aligned on narrow cards`
- `docs: document signing policy`

## Licencias y procedencia

- Al contribuir aceptas que tu aporte se distribuya bajo la licencia MIT del proyecto.
- No envíes código con licencias incompatibles.
- Conserva los avisos de copyright y las licencias de terceros cuando corresponda.
- Consulta [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
