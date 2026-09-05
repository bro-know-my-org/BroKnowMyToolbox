import {
  createMemoryHistory,
  createRouter,
  createWebHashHistory,
  type RouterHistory,
} from "vue-router";

import { registeredTools } from "../tools/registry";
import HomeView from "../views/home/index.vue";

function defaultHistory(): RouterHistory {
  return import.meta.env.MODE === "test"
    ? createMemoryHistory()
    : createWebHashHistory();
}

export function createAppRouter(history = defaultHistory()) {
  return createRouter({
    history,
    routes: [
      { path: "/", name: "Home", component: HomeView },
      ...registeredTools.map((tool) => ({
        path: tool.route,
        name: tool.routeName,
        component: tool.component,
      })),
      { path: "/:pathMatch(.*)*", redirect: "/" },
    ],
  });
}
