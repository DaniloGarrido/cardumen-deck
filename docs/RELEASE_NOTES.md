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

- Esta versión preliminar no está firmada digitalmente. Windows SmartScreen puede mostrar una advertencia.
- Verifica el SHA256 antes de ejecutar.
- No hay actualización automática; descarga una versión nueva manualmente.
- La aplicación no tiene telemetría ni backend remoto. Consulta [PRIVACY.md](https://github.com/DaniloGarrido/cardumen-deck/blob/v0.1.0/PRIVACY.md).
- La firma con SignPath Foundation se aplicará en una versión posterior. Consulta la [Code signing policy](https://github.com/DaniloGarrido/cardumen-deck/blob/main/CODE_SIGNING.md).
- Free code signing provided by SignPath.io, certificate by SignPath Foundation.
- Consulta los límites de seguridad en [SECURITY.md](https://github.com/DaniloGarrido/cardumen-deck/blob/v0.1.0/SECURITY.md).
- El uso de inteligencia artificial en el desarrollo se documenta en [AI.md](https://github.com/DaniloGarrido/cardumen-deck/blob/v0.1.0/AI.md).
- Consulta el aviso legal en [LEGAL.md](https://github.com/DaniloGarrido/cardumen-deck/blob/v0.1.0/LEGAL.md).

### Verificación

```powershell
Get-FileHash .\CardumenDeck-v0.1.0-windows-x64-portable.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```
