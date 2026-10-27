# Arquitectura

## Vista general

Cardumen Deck es una aplicación Tauri 2 con interfaz Vue 3 y backend Rust.

```text
Interfaz Vue 3
  -> comandos Tauri (invoke)
    -> módulos Rust
      -> procesos locales, archivos, Git, registros y métricas
```

No existe un backend remoto. La aplicación lee y escribe archivos locales.

## Interfaz

- `src/App.vue`: composición principal y estado de los modales.
- `src/components/`: tarjetas, listas, formularios, modales y paneles.
- `src/composables/`: servicios, espacios de trabajo, Git, tema, registros, uso y notificaciones.
- `src/types/`: contratos TypeScript compartidos.
- `src/style.css`: tokens visuales, temas y estilos globales.

## Backend

- `commands.rs`: comandos expuestos a la interfaz.
- `config.rs`: configuración persistente.
- `process.rs`: creación y control de procesos.
- `runtime.rs`: estado de ejecución y reconciliación.
- `metrics.rs`: CPU, RAM y procesos.
- `health.rs`: verificaciones de estado HTTP/TCP/proceso.
- `logs.rs` y `dashboard_log.rs`: registros de servicios y de la aplicación.
- `git.rs`: operaciones Git.
- `espacios de trabajo.rs`: espacios de trabajo y orden de servicios.
- `dependencies.rs` y `environment.rs`: detección de entorno.

## Datos

- Configuración: `%APPDATA%\CardumenDeck\config.json`
- Estado de ejecución: `%APPDATA%\CardumenDeck\runtime.json`
- Registros (logs): `%LOCALAPPDATA%\CardumenDeck\logs\`
- Migración heredada: copia `config.json`, `runtime.json` y el registro del dashboard desde `ServiceDashboard` si `CardumenDeck` aún no existe.

## Límites de confianza

- La aplicación se ejecuta con los privilegios del usuario de Windows.
- Los comandos de los servicios se ejecutan con esos mismos privilegios.
- No hay un entorno aislado (sandbox) de seguridad.
- El Job Object se usa para administrar procesos, no para aislarlos.
- Los comandos, los repositorios, las URLs y los puntos de conexión configurados no se consideran confiables por defecto.
- La configuración y los registros no están cifrados.

## Ciclo de vida de procesos

La aplicación crea procesos locales, registra su PID y estado, y reconcilia procesos huérfanos al iniciar. Los servicios se pueden adoptar si siguen vivos.

## Verificaciones de estado

El monitor se ejecuta en segundo plano y actualiza el estado de salud. Las verificaciones pueden ser:

- HTTP: respuesta menor a 500.
- TCP: conexión al puerto.
- Proceso: PID vivo.

## Red

| Acción | Destino | Datos enviados | Cuándo ocurre |
|---|---|---|---|
| Verificación HTTP | URL configurada | `GET` con `User-Agent: CardumenDeck/0.1` | Mientras la verificación está activa |
| Verificación TCP | Host y puerto configurados | Conexión TCP | Mientras la verificación está activa |
| Operaciones Git | Remoto configurado | Datos estándar de Git | Cuando el usuario ejecuta `fetch`, `pull` o `push` |
| Abrir URL | Navegador predeterminado | La URL solicitada | Cuando el usuario abre una URL |
| WebView2/Windows | Servicios del sistema | Datos según la configuración del sistema | Fuera del control de la aplicación |

La aplicación no agrega credenciales a las verificaciones HTTP. Git puede usar Git Credential Manager u otro asistente de credenciales del sistema; la aplicación no almacena esas credenciales.

## Datos en reposo

| Dato | Ubicación | Cifrado | Observación |
|---|---|---|---|
| Configuración | `%APPDATA%\CardumenDeck\config.json` | No | Incluye variables de entorno |
| Estado de ejecución | `%APPDATA%\CardumenDeck\runtime.json` | No | PIDs, estados y rutas |
| Registros | `%LOCALAPPDATA%\CardumenDeck\logs\` | No | Pueden contener secretos |
| Perfil de WebView2 | `%LOCALAPPDATA%\CardumenDeck\` | Según el sistema | Tema, vista y preferencias |

## Telemetría

Cardumen Deck no incluye telemetría, analíticas, reportes remotos de errores ni un backend propio. Windows, WebView2, Git y los puntos de conexión configurados pueden tener sus propias políticas y conexiones.

Consulta [PRIVACY.md](../PRIVACY.md) y [SECURITY.md](../SECURITY.md).

## Seguridad

- La aplicación ejecuta comandos configurados por el usuario.
- No hay telemetría ni conexión a un backend propio.
- Las verificaciones de estado solo consultan puntos de conexión configurados y Git contacta el remoto cuando el usuario ejecuta una operación.
- Las versiones se firman cuando SignPath Foundation lo permita.
- La ventana usa una Política de seguridad de contenido (CSP) restrictiva; el script de tema vive en `FrontEnd/public/theme-init.js`.
- Los secretos y certificados no se versionan.
