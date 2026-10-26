# Contribuir

Gracias por querer contribuir a Cardumen Deck.

## Antes de empezar

- Busca issues existentes antes de abrir uno nuevo.
- Para cambios grandes, abre un issue primero para acordar el enfoque.
- No incluyas secretos, tokens, certificados ni rutas personales.

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

Antes de abrir un PR:

```powershell
npm run lint
npm --workspace FrontEnd run check
npm --workspace FrontEnd run build
cargo fmt --manifest-path Backend/Cargo.toml -- --check
cargo clippy --manifest-path Backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path Backend/Cargo.toml --lib
```

## Pull requests

- Mantén el cambio enfocado.
- Explica qué problema resuelve.
- Incluye pasos para probarlo.
- Adjunta captura o video si cambia la interfaz.
- Actualiza la documentación si cambia el comportamiento.

## Commits

Usa mensajes claros y en imperativo. Ejemplos:

- `feat: add service health filters`
- `fix: keep service actions aligned on narrow cards`
- `docs: document release signing`

## Licencia

Al contribuir aceptas que tu aporte se distribuya bajo la licencia MIT del proyecto.