import {createRouter, createWebHashHistory} from "vue-router";
import LegCpu from "./components/LegCpu.vue";

export const routes = [
    {path: '/', component: LegCpu},
]

export const router = createRouter({
    history: createWebHashHistory(),
    routes,
})
