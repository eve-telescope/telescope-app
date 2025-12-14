import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import AboutWindow from './components/AboutWindow.vue'

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
    ],
})

export default router
