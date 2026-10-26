# Arquitectura

## Vista general

Cardumen Deck es una aplicación Tauri 2 con frontend Vue 3 y backend Rust.

```text
Vue 3 UI
  -> comandos Tauri (invoke)
    -> módulos Rust
      -> procesos locales, archivos, Git, logs y métricas
```

No existe backend remoto. La app lee y escribe archivos locales.

## Frontend

- `src/App.vue`: composición principal y estado de modales.
- `src/components/`: tarjetas, listas, formularios, modales y paneles.
- `src/composables/`: servicios, workspaces, Git, tema, logs, uso y notificaciones.
- `src/types/`: contratos TypeScript compartidos.
- `src/style.css`: tokens visuales, temas y estilos globales.

## Backend

- `commands.rs`: comandos expuestos al frontend.
- `config.rs`: configuración persistente.
- `process.rs`: creación y control de procesos.
- `runtime.rs`: estado de ejecución y reconciliación.
- `metrics.rs`: CPU, RAM y procesos.
- `health.rs`: health checks HTTP/TCP/proceso.
- `logs.rs` y `dashboard_log.rs`: logs de servicios y de la app.
- `git.rs`: operaciones Git.
- `workspaces.rs`: workspaces y orden de servicios.
- `dependencies.rs` y `environment.rs`: detección de entorno.

## Datos

- Configuración: `%APPDATA%\CardumenDeck\config.json`
- Runtime: `%APPDATA%\CardumenDeck\runtime.json`
- Logs: `%LOCALAPPDATA%\CardumenDeck\logs\`
- Migración legacy: copia `config.json`, `runtime.json` y el dashboard log desde `ServiceDashboard` si `CardumenDeck` aún no existe.

## Ciclo de vida de procesos

La app crea procesos locales, registra su PID y estado, y reconcilia procesos huérfanos al iniciar. Los servicios se pueden adoptar si siguen vivos.

## Health checks

El monitor corre en segundo plano y actualiza el estado de salud. Los checks pueden ser:

- HTTP: respuesta menor a 500.
- TCP: conexión al puerto.
- Proceso: PID vivo.

## Seguridad

- La app ejecuta comandos configurados por el usuario.
- No hay telemetría ni conexión a un backend propio. Los health checks solo consultan endpoints configurados y Git contacta el remoto cuando el usuario ejecuta una operación.
- Las releases se firman cuando SignPath Foundation lo permita.
- La ventana usa una CSP restrictiva; el script de tema vive en FrontEnd/public/theme-init.js.
- Los secretos y certificados no se versionan.