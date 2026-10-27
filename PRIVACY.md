# Política de privacidad

> Versión en inglés: [PRIVACY.en.md](PRIVACY.en.md). La versión en español es la principal. En caso de conflicto, prevalece esta versión. [LICENSE](LICENSE) prevalece sobre este documento.

## Alcance

Esta política describe qué datos usa Cardumen Deck, dónde se almacenan y qué conexiones de red puede realizar. Se aplica a la aplicación de escritorio y a su configuración local.

## Resumen

- Cardumen Deck **no incluye telemetría, analíticas, reportes de errores remotos ni un backend propio**.
- No hay cuentas, inicio de sesión, sincronización en la nube ni publicidad.
- Los datos de la aplicación se guardan localmente.
- La aplicación solo usa la red para acciones concretas: verificaciones de estado, operaciones Git y apertura de URLs.
- Windows, WebView2, Git y los servicios que ejecutes pueden tener sus propias políticas y conexiones.

## Qué no hace la aplicación

Cardumen Deck no:

- envía telemetría ni métricas de uso;
- usa SDK de analíticas o publicidad;
- envía reportes de errores a un servidor propio;
- crea identificadores de usuario;
- sincroniza datos con la nube;
- descarga configuración remota;
- busca actualizaciones automáticamente;
- vende ni comparte datos con terceros.

## Datos locales

| Dato | Ubicación | Finalidad | Cifrado |
|---|---|---|---|
| Configuración | `%APPDATA%\CardumenDeck\config.json` | Servicios, espacios de trabajo, ajustes y variables de entorno | No |
| Estado de ejecución | `%APPDATA%\CardumenDeck\runtime.json` | PIDs, estados, rutas de registros y marcas de tiempo | No |
| Registros de servicios | `%LOCALAPPDATA%\CardumenDeck\logs\` | Comando, argumentos, carpeta de trabajo, salida y errores | No |
| Perfil de WebView2 | `%LOCALAPPDATA%\CardumenDeck\` | Tema, vista, barra lateral y preferencias locales | Según WebView2/Windows |
| Portapapeles | Memoria del sistema | Leer o escribir texto cuando el usuario lo solicita | No aplica |

Las variables de entorno definidas para un servicio se guardan en `config.json` **sin cifrar**. No se recomienda guardar secretos de producción en ese archivo.

Los registros pueden contener información sensible: comandos, rutas, nombres de servicios, salida de procesos y errores. También pueden incluir secretos si un servicio los imprime. Revisa y protege los registros antes de compartirlos.

## Conexiones de red

| Acción | Destino | Datos enviados | Cuándo ocurre |
|---|---|---|---|
| Verificación HTTP | URL configurada para el servicio | Solicitud `GET` con `User-Agent: CardumenDeck/0.1` | Mientras la verificación de estado está activa |
| Verificación TCP | Host y puerto configurados | Conexión TCP | Mientras la verificación de estado está activa |
| Operaciones Git | Remoto configurado en el repositorio | Datos estándar de Git | Cuando el usuario ejecuta `fetch`, `pull` o `push` |
| Abrir URL | Navegador predeterminado | La URL solicitada | Cuando el usuario abre una URL |
| WebView2/Windows | Servicios del sistema operativo | Datos según la configuración de Windows/WebView2 | Fuera del control de Cardumen Deck |

La aplicación no agrega credenciales a las verificaciones HTTP. Si una URL contiene tokens o credenciales en la ruta o en la cadena de consulta, esos datos pueden enviarse al punto de conexión configurado.

Git puede usar Git Credential Manager u otro asistente de credenciales del sistema. Cardumen Deck no almacena esas credenciales.

## Terceros

- **WebView2 y Windows** pueden recopilar diagnósticos o telemetría según su propia configuración.
- **Git y los remotos configurados** tienen sus propias políticas.
- **Los puntos de conexión de verificación de estado** reciben las solicitudes que el usuario configure.
- **GitHub** aloja el repositorio, los flujos de trabajo y las versiones; no recibe datos de la aplicación en tiempo de ejecución.

## Retención y eliminación

- Los registros se conservan según la configuración de retención de Cardumen Deck.
- `config.json` y `runtime.json` permanecen hasta que el usuario los elimine.
- Para eliminar los datos locales, cierra la aplicación y elimina:
  - `%APPDATA%\CardumenDeck`
  - `%LOCALAPPDATA%\CardumenDeck`

No hay copia de seguridad remota ni recuperación por parte de los mantenedores.

## Seguridad de los datos locales

Cardumen Deck no cifra su configuración ni sus registros. La protección depende del sistema operativo, los permisos del usuario y el cifrado del dispositivo.

Recomendaciones:

- usar una cuenta de Windows con privilegios mínimos;
- mantener el cifrado del dispositivo activo, por ejemplo BitLocker;
- no guardar secretos de producción en `config.json`;
- revisar los registros antes de compartirlos;
- mantener copias de seguridad de la configuración si es importante para tu flujo de trabajo.

## Uso en desarrollo real

Cardumen Deck es una herramienta local para desarrollo. No es un servicio gestionado ni un componente de producción. Puedes usarla en tu flujo real de desarrollo, pero eres responsable de los comandos que ejecutes, de las credenciales que uses, de los datos que guardes y de las copias de seguridad.

Si la usas en un dispositivo de una empresa o con código de una empresa, pueden aplicarse las políticas internas de esa empresa.

## Cambios en esta política

Esta política puede actualizarse cuando cambie el comportamiento de la aplicación. Los cambios relevantes se registrarán en [CHANGELOG.md](CHANGELOG.md).

## Contacto

Para preguntas de privacidad, usa las discusiones del repositorio. Para vulnerabilidades, usa GitHub Security Advisories. No publiques secretos ni datos personales en incidencias públicas.

Este documento no constituye asesoría legal. Consulta [LEGAL.md](LEGAL.md).
