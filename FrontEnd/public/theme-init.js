(() => {
  const themeKey = 'cardumen-deck-theme'
  const paletteKey = 'cardumen-deck-palette'
  const palettes = ['indigo', 'blue', 'cyan', 'purple', 'red', 'orange', 'gray']
  const themeColors = {
    indigo: { light: '#f8fafc', dark: '#0b1120' },
    blue: { light: '#f3f8ff', dark: '#07111f' },
    cyan: { light: '#f2fdff', dark: '#06151a' },
    purple: { light: '#faf7ff', dark: '#120b1e' },
    red: { light: '#fff7f7', dark: '#160b0f' },
    orange: { light: '#fff8f1', dark: '#160d06' },
    gray: { light: '#f7f8fa', dark: '#0c0f14' },
  }
  const storedTheme = localStorage.getItem(themeKey)
  const storedPalette = localStorage.getItem(paletteKey)
  const theme = storedTheme === 'light' || storedTheme === 'dark'
    ? storedTheme
    : (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
  const palette = palettes.includes(storedPalette) ? storedPalette : 'indigo'
  document.documentElement.dataset.theme = theme
  document.documentElement.dataset.palette = palette
  document.documentElement.style.colorScheme = theme
  document
    .querySelector('meta[name="theme-color"]')
    ?.setAttribute('content', themeColors[palette][theme])
})()
