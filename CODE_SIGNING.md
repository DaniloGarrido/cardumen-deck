# Política de firma de código

## Estado actual

Las versiones se generan desde GitHub Actions y actualmente pueden publicarse sin firma digital. La versión inicial `0.1.0` puede no estar firmada.

## Objetivo

El proyecto busca ser aceptado en **SignPath Foundation**, que ofrece firma de código gratuita para proyectos de código abierto.

Si se aprueba:

- El editor que verán los usuarios será **SignPath Foundation**.
- La firma se realizará dentro del flujo de trabajo de GitHub Actions.
- Las claves privadas nunca se almacenarán en el repositorio.

## Reglas

- Solo se firman binarios construidos desde confirmaciones (commits) de este repositorio.
- No se firman artefactos de terceros.
- No se suben certificados, tokens ni claves privadas.
- Cada versión incluye `SHA256SUMS.txt` para la versión portable y el instalador NSIS.
- La compilación debe ser reproducible desde una etiqueta de Git.

## Verificación

```powershell
signtool verify /pa /v CardumenDeck-v0.1.0-windows-x64-portable.exe
signtool verify /pa /v CardumenDeck-v0.1.0-windows-x64-setup.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.0-windows-x64-portable.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.0-windows-x64-setup.exe
```

## Sumas de verificación

Cada versión adjunta `SHA256SUMS.txt` para verificar la integridad de la versión portable y del instalador.

## Checklist para SignPath

- [x] Licencia OSI aprobada (MIT).
- [x] Repositorio público.
- [x] README y documentación de compilación.
- [x] Política de firma creada (este archivo).
- [ ] Versiones construidas desde GitHub Actions (portable + instalador).
- [ ] Aplicación enviada y aprobada por SignPath Foundation.
- [ ] Autenticación multifactor (MFA) activa para los mantenedores.
- [ ] Primera versión firmada.
