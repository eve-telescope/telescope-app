import { createApp } from 'vue'
import RouterRoot from './RouterRoot.vue'
import router from './router'
import './styles/global.css'
import { initLogger } from './utils/logger'

initLogger()

createApp(RouterRoot).use(router).mount('#app')
