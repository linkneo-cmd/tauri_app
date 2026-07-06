import { createRouter, createWebHashHistory } from "vue-router";
import HomePage from "./views/HomePage.vue";
import ImageBatchPage from "./views/ImageBatchPage.vue";

const routes = [
  { path: "/", name: "home", component: HomePage },
  { path: "/image-batch", name: "image-batch", component: ImageBatchPage },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
