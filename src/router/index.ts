import Layout from "../layout/index.vue";

import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";

export const constantRoutes: RouteRecordRaw[] = [
  /** 默认路由 */
  {
    path: "/",
    component: Layout,
    name: "RootLayout",
    redirect: "/index",
    children: [
      {
        path: "index",
        name: "Index",
        component: () => import("../views/home/index.vue"),
      },
      {
        path: "settings",
        name: "Settings",
        component: () => import("../views/settings/index.vue"),
      },
      {
        path: ":pathMatch(.*)*",
        name: "NotFound",
        component: () => import("../views/error/404/index.vue"),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes: constantRoutes,
});

export default router;
