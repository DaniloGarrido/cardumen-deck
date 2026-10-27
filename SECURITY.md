# Seguridad

> Versión en inglés: [SECURITY.en.md](SECURITY.en.md). La versión en español es la principal. En caso de conflicto, prevalece esta versión. [LICENSE](LICENSE) prevalece sobre este documento.

## Versiones soportadas

La rama principal y la última versión `0.1.x` reciben correcciones de seguridad. No existe un acuerdo de nivel de servicio (SLA) ni soporte comercial.

## Reportar una vulnerabilidad

No abras una incidencia pública con detalles explotables.

Usa **GitHub Security Advisories** del repositorio:

1. Ve a la pestaña **Security**.
2. Entra en **Advisories**.
3. Crea un reporte privado.

Incluye:

- versión afectada;
- pasos para reproducir;
- impacto esperado;
- entorno (Windows, WebView2, versión de la aplicación);
- registros relevantes sin datos personales ni secretos.

La divulgación es coordinada y se realiza con esfuerzo razonable. No se garantizan plazos de respuesta.

## Alcance

El alcance incluye:

- código de la aplicación;
- configuración de Tauri;
- Política de seguridad de contenido (CSP);
- capacidades (capabilities);
- flujos de trabajo de CI y publicación;
- configuración de dependencias y versiones.

Quedan fuera del alcance:

- Windows, WebView2 y otros componentes del sistema;
- Git y los remotos configurados;
- servicios, comandos y puntos de conexión configurados por el usuario;
- dependencias de terceros, salvo que el problema sea causado por el uso que hace Cardumen Deck de ellas;
- equipos comprometidos o cuentas de usuario comprometidas.

## Postura de seguridad

Cardumen Deck implementa las siguientes medidas:

- Política de seguridad de contenido (CSP) restrictiva en la ventana de Tauri.
- Capacidades (capabilities) acotadas para las funciones que usa la interfaz.
- Sin backend remoto ni telemetría.
- Datos almacenados localmente.
- Archivos de bloqueo (lockfiles) para dependencias de Rust y npm.
- Dependabot, alertas de seguridad y actualizaciones de seguridad configuradas.
- Escaneo de secretos y protección de envío (push protection) configurados en GitHub.
- CodeQL configurado para JavaScript/TypeScript.
- CI configurado para lint, verificación de tipos, compilación, formato, Clippy y pruebas.
- Sumas de verificación SHA256 en las versiones.
- Firma de código pendiente de aprobación por SignPath Foundation.

## Lo que no se afirma

Cardumen Deck no afirma ni garantiza:

- haber sido auditado por un tercero;
- estar certificado bajo un estándar de seguridad;
- estar libre de vulnerabilidades;
- ofrecer un entorno aislado (sandbox) de seguridad;
- cifrar la configuración o los registros en reposo;
- proteger contra comandos maliciosos ejecutados por el usuario;
- proteger contra dependencias comprometidas;
- proteger contra un Windows, WebView2 o Git comprometidos;
- ofrecer un acuerdo de nivel de servicio (SLA), soporte comercial o programa de recompensas por errores;
- indemnizar al usuario o a terceros.

## Modelo de confianza

- La aplicación se ejecuta con los privilegios del usuario de Windows.
- Los comandos de los servicios se ejecutan con esos mismos privilegios.
- Los comandos pueden acceder a archivos, credenciales y redes a las que el usuario tenga acceso.
- El Job Object se usa para administrar procesos, no como entorno aislado (sandbox) de seguridad.
- Las operaciones Git pueden modificar el repositorio y el remoto.
- Las verificaciones de estado pueden alcanzar puntos de conexión internos o externos configurados.
- Los registros y la configuración pueden contener datos sensibles.

## Datos y secretos

- `config.json` guarda las variables de entorno sin cifrar.
- Los registros pueden contener comandos, rutas, salida de procesos y secretos impresos por un servicio.
- No se recomienda usar secretos de producción en la configuración.
- Git Credential Manager u otro asistente de credenciales del sistema puede administrar credenciales de Git; Cardumen Deck no las almacena.
- Se recomienda cifrado de dispositivo, cuenta con privilegios mínimos y copias de seguridad.

Consulta [PRIVACY.md](PRIVACY.md) para el detalle de datos y red.

## Red

La aplicación solo usa la red para:

- verificaciones de estado HTTP/TCP configuradas;
- operaciones Git contra remotos configurados;
- apertura de URLs en el navegador predeterminado.

Windows, WebView2, Git y los puntos de conexión configurados pueden tener sus propias políticas y conexiones.

## Integridad de las versiones

- Cada versión está configurada para publicar portable, instalador NSIS y `SHA256SUMS.txt`.
- La firma de código con SignPath Foundation está pendiente.
- Mientras no haya firma, Windows SmartScreen puede mostrar advertencias.
- Verifica el SHA256 antes de ejecutar un binario descargado.

Consulta [CODE_SIGNING.md](CODE_SIGNING.md).

## Cadena de suministro

- Se usan archivos de bloqueo (lockfiles) para dependencias.
- Dependabot está configurado para npm, Cargo y GitHub Actions.
- El escaneo de secretos y la protección de envío (push protection) están activados.
- CodeQL está configurado para JavaScript/TypeScript.
- La aplicación se distribuye como binario de Windows y no incluye telemetría.

Estas medidas reducen riesgos, pero no garantizan que una dependencia o un flujo de trabajo sea seguro.

## Uso en desarrollo real

Cardumen Deck es una herramienta local para desarrollo. No es un servicio gestionado ni un componente de producción. Puedes usarla en tu flujo real de desarrollo, pero eres responsable de los comandos, credenciales, datos y copias de seguridad.

Si la usas en un dispositivo de una empresa o con código de una empresa, pueden aplicarse las políticas internas de esa empresa.

## Soporte

El soporte es comunitario y de mejor esfuerzo. No hay acuerdo de nivel de servicio (SLA) ni soporte comercial. Consulta [SUPPORT.md](SUPPORT.md).

Este documento no constituye asesoría legal. Consulta [LEGAL.md](LEGAL.md).
