## Cardumen Deck v0.1.0

Primera versión preliminar pública de Cardumen Deck: un panel local para iniciar, monitorear y operar servicios de desarrollo en Windows.

### Descargas

- `CardumenDeck-v0.1.0-windows-x64-portable.exe`: no requiere instalación.
- `CardumenDeck-v0.1.0-windows-x64-setup.exe`: instalador NSIS.
- `SHA256SUMS.txt`: sumas de verificación para ambos archivos.

### Requisitos

- Windows 10 u 11.
- WebView2 Runtime.
- Git para Windows si vas a usar las funciones Git.

### Notas

- Esta versión puede estar sin firma digital. Windows SmartScreen puede mostrar una advertencia.
- Verifica el SHA256 antes de ejecutar.
- No hay actualización automática; descarga una versión nueva manualmente.
- La aplicación no tiene telemetría ni backend remoto. Consulta [PRIVACY.md](../PRIVACY.md).
- La firma con SignPath Foundation está pendiente de aprobación.
- Consulta los límites de seguridad en [SECURITY.md](../SECURITY.md).
- El uso de inteligencia artificial en el desarrollo se documenta en [AI.md](../AI.md).
- Consulta el aviso legal en [LEGAL.md](../LEGAL.md).

### Verificación

```powershell
Get-FileHash .\CardumenDeck-v0.1.0-windows-x64-portable.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```
