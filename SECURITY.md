# Seguridad

## Versiones soportadas

La rama principal y la última release `0.1.x` reciben correcciones de seguridad.

## Reportar una vulnerabilidad

No abras un issue público con detalles explotables.

Usa **GitHub Security Advisories** del repositorio:

1. Ve a la pestaña **Security**.
2. Entra en **Advisories**.
3. Crea un reporte privado.

Incluye:

- Versión afectada.
- Pasos para reproducir.
- Impacto esperado.
- Entorno (Windows, WebView2, versión de la app).
- Logs relevantes sin datos personales.

## Alcance

Cardumen Deck ejecuta procesos locales y operaciones Git. Los puntos de mayor riesgo son:

- Ejecución de comandos y procesos configurados por el usuario.
- Lectura/escritura de archivos y logs locales.
- Operaciones Git destructivas.
- Enlaces y rutas abiertas desde la interfaz.

La aplicación no tiene backend remoto ni telemetría. Los health checks solo consultan las URLs o puertos configurados y las operaciones Git contactan el remoto solo cuando el usuario las ejecuta.

## Firma de código

Las releases se publican desde GitHub Actions. La release `0.1.x` puede estar sin firma digital y la firma con SignPath Foundation está pendiente de aprobación. Ver [CODE_SIGNING.md](CODE_SIGNING.md).

## Content Security Policy

La ventana de Tauri usa una CSP restrictiva para scripts, estilos, imágenes y conexiones IPC. Los scripts inline fueron movidos a un archivo local para evitar `unsafe-inline` en `script-src`.
