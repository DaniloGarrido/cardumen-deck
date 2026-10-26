# Política de firma de código

## Estado actual

Las releases se generan desde GitHub Actions y actualmente pueden publicarse sin firma digital. La release inicial `0.1.0` puede ser unsigned.

## Objetivo

El proyecto busca ser aceptado en **SignPath Foundation**, que ofrece firma de código gratuita para proyectos open source.

Si se aprueba:

- El publisher que verán los usuarios será **SignPath Foundation**.
- La firma se realizará dentro del pipeline de GitHub Actions.
- Las claves privadas nunca se almacenarán en el repositorio.

## Reglas

- Solo se firman binarios construidos desde commits de este repositorio.
- No se firman artefactos de terceros.
- No se suben certificados, tokens ni claves privadas.
- Cada release incluye `SHA256SUMS.txt` para el portable y el instalador NSIS.
- La build debe ser reproducible desde un tag de Git.

## Verificación

```powershell
signtool verify /pa /v CardumenDeck-v0.1.0-windows-x64-portable.exe
signtool verify /pa /v CardumenDeck-v0.1.0-windows-x64-setup.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.0-windows-x64-portable.exe
Get-AuthenticodeSignature .\CardumenDeck-v0.1.0-windows-x64-setup.exe
```

## Checksums

Cada release adjunta `SHA256SUMS.txt` para verificar la integridad del portable y del instalador.

## Checklist para SignPath

- [x] Licencia OSI aprobada (MIT).
- [ ] Repositorio publico.
- [x] README y documentacion de build.
- [x] Politica de firma creada (este archivo).
- [ ] Releases construidas desde GitHub Actions (portable + instalador).
- [ ] Aplicacion enviada y aprobada por SignPath Foundation.
- [ ] MFA activo para los maintainers.
- [ ] Primer release firmado.
