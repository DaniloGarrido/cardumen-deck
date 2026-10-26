## Qué cambia

<!-- Describe el cambio en una o dos frases. -->

## Por qué

<!-- Explica el problema o la motivación. -->

## Cómo probarlo

<!-- Pasos concretos para validar el cambio. -->

## Checklist

- [ ] `npm run lint`
- [ ] `npm --workspace FrontEnd run check`
- [ ] `npm --workspace FrontEnd run build`
- [ ] `cargo fmt --manifest-path Backend/Cargo.toml -- --check`
- [ ] `cargo clippy --manifest-path Backend/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path Backend/Cargo.toml --lib`
- [ ] Probado en Windows
- [ ] Sin secretos ni rutas personales
- [ ] Documentación actualizada si aplica