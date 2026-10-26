import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { logDashboardError } from './composables/useDashboardLog'

const app = createApp(App)

app.config.errorHandler = (error, _instance, info) => {
  void logDashboardError('vue.error', error, {
    title: 'Error interno de la interfaz',
    details: info,
    notify: true,
  })
}

window.addEventListener('error', (event) => {
  if (!event.error) return
  void logDashboardError('window.error', event.error, {
    title: 'Error no controlado',
    notify: true,
  })
})

window.addEventListener('unhandledrejection', (event) => {
  void logDashboardError('window.unhandledrejection', event.reason, {
    title: 'Promesa rechazada sin controlar',
    notify: true,
  })
})

app.mount('#app')
