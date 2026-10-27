## Cardumen Deck v0.1.0

Primera preview pública de Cardumen Deck: un dashboard local para arrancar, monitorear y operar servicios de desarrollo en Windows.

### Descargas

- `CardumenDeck-v0.1.0-windows-x64-portable.exe`: no requiere instalación.
- `CardumenDeck-v0.1.0-windows-x64-setup.exe`: instalador NSIS.
- `SHA256SUMS.txt`: hashes para verificar ambos archivos.

### Requisitos

- Windows 10 u 11.
- WebView2 Runtime.
- Git para Windows si vas a usar las funciones Git.

### Notas

- Esta release puede estar sin firma digital. Windows SmartScreen puede mostrar una advertencia.
- Verifica el SHA256 antes de ejecutar.
- No hay auto-update; descarga una release nueva manualmente.
- La app no tiene telemetría ni backend remoto.
- La firma con SignPath Foundation está pendiente de aprobación.

### Verificación

```powershell
Get-FileHash .\CardumenDeck-v0.1.0-windows-x64-portable.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```
