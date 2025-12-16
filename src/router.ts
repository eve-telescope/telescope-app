import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import AboutWindow from './components/AboutWindow.vue'
import OverlayWindow from './components/OverlayWindow.vue'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/',
            component: App,
        },
        {
            path: '/about',
            component: AboutWindow,
        },
        {
            path: '/overlay',
            component: OverlayWindow,
        },
    ],
})

export default router
