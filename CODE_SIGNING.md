# Code signing policy

> English version: [CODE_SIGNING.en.md](CODE_SIGNING.en.md). La versión en español es la autoritativa. En caso de conflicto, prevalece la versión en español.

## Estado actual

Las versiones se generan desde GitHub Actions. La versión `0.1.0` se publicó sin firma digital como paso previo a la incorporación de SignPath Foundation. La firma se aplicará desde la versión `0.1.1`.

Release `v0.1.0`: <https://github.com/DaniloGarrido/cardumen-deck/releases/tag/v0.1.0>.

## Declaración

Las versiones firmadas se publicarán con firma de código gratuita proporcionada por [SignPath.io](https://about.signpath.io), con certificado de [SignPath Foundation](https://signpath.org).

> Free code signing provided by SignPath.io, certificate by SignPath Foundation.

El editor que verán los usuarios en las versiones firmadas será **SignPath Foundation**.

## Alcance

- Solo se firman binarios construidos desde el código fuente de este repositorio.
- Se firman el ejecutable portable y el instalador NSIS.
- No se firman artefactos de terceros.
- El instalador puede incluir componentes de terceros sin firma, permitidos por la política de SignPath Foundation.
- No se suben certificados, tokens ni claves privadas al repositorio.
- Cada versión incluye `SHA256SUMS.txt` para verificar la integridad de los archivos publicados.

## Roles

- **Autor:** [@DaniloGarrido](https://github.com/DaniloGarrido), propietario del repositorio. Puede modificar el código sin revisión adicional.
- **Revisor:** [@DaniloGarrido](https://github.com/DaniloGarrido), propietario del repositorio. Revisa los cambios propuestos por personas que no son mantenedoras.
- **Aprobador:** [@DaniloGarrido](https://github.com/DaniloGarrido), propietario del repositorio. Aprueba manualmente cada solicitud de firma.

Cada solicitud de firma requiere aprobación manual. No hay firma automática de releases.

## Privacidad

La política de privacidad está en [PRIVACY.md](PRIVACY.md). Cardumen Deck no tiene telemetría, analíticas, reportes remotos de errores ni backend propio.

## Requisitos de compilación

- La compilación se ejecuta en GitHub Actions sobre runners hospedados por GitHub (`windows-latest`).
- La versión se construye desde una etiqueta `v*` que coincide con la versión de la aplicación.
- El ejecutable principal se firma antes de generar el instalador NSIS.
- El instalador se firma después de generarse.
- Los artefactos sin firma se suben como artifacts de GitHub Actions antes de enviarlos a SignPath.
- La compilación y el flujo de firma quedan registrados en el historial de GitHub Actions.

## Metadatos de los binarios

Los binarios firmados deben tener metadatos coherentes:

- Nombre de producto: `Cardumen Deck`.
- Versión de producto: la misma que la etiqueta de la release.
- Versión de archivo: la misma que la versión de producto.

La configuración de artefactos de SignPath debe exigir estos valores antes de firmar.

## Verificación

```powershell
signtool verify /pa /v CardumenDeck-v0.1.1-windows-x64-portable.exe
signtool verify /pa /v CardumenDeck-v0.1.1-windows-x64-setup.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.1-windows-x64-portable.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.1-windows-x64-setup.exe
```

En una versión firmada, `Get-AuthenticodeSignature` debe mostrar un estado `Valid` y el editor debe incluir **SignPath Foundation**.

## Checklist para SignPath

- [x] Licencia OSI aprobada (MIT).
- [x] Repositorio público.
- [x] README y documentación de compilación.
- [x] Política de firma creada (este archivo).
- [x] Versiones construidas desde GitHub Actions (portable + instalador).
- [ ] Aplicación enviada y aprobada por SignPath Foundation.
- [ ] Autenticación multifactor (MFA) activa para los mantenedores en GitHub y SignPath.
- [ ] Primera versión firmada.
