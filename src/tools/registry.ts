import { h } from "vue";
import { NIcon } from "naive-ui";
import type { Component } from "vue";
import type { MenuOption } from "naive-ui";
import type { Router, RouteRecordRaw } from "vue-router";
import { ClipboardTextLtr24Regular, DocumentAdd24Regular, DataUsage24Regular } from "@vicons/fluent";

type ToolComponent = NonNullable<RouteRecordRaw["component"]>;

export interface ToolDefinition {
  id: string;
  titleKey: string;
  descriptionKey: string;
  path: string;
  routeName: string;
  icon?: Component;
  component: ToolComponent;
}

export const tools: ToolDefinition[] = [
  {
    id: "file-creator",
    titleKey: "tools.fileCreator.title",
    descriptionKey: "tools.fileCreator.registryDescription",
    path: "/tools/file-creator",
    routeName: "ToolFileCreator",
    icon: DocumentAdd24Regular,
    component: () => import("../views/tools/file-creator/index.vue"),
  },
  {
    id: "clipboard",
    titleKey: "tools.clipboard.title",
    descriptionKey: "tools.clipboard.registryDescription",
    path: "/tools/clipboard",
    routeName: "ToolClipboard",
    icon: ClipboardTextLtr24Regular,
    component: () => import("../views/tools/clipboard/index.vue"),
  },
  {
    id: "spark-analyzer",
    titleKey: "tools.sparkAnalyzer.title",
    descriptionKey: "tools.sparkAnalyzer.registryDescription",
    path: "/tools/spark-analyzer",
    routeName: "ToolSparkAnalyzer",
    icon: DataUsage24Regular,
    component: () => import("../views/tools/spark-analyzer/index.vue"),
  },
];

export function createToolRoutes(): RouteRecordRaw[] {
  return tools.map((tool) => ({
    path: tool.path.replace(/^\//, ""),
      name: tool.routeName,
      component: tool.component,
      meta: {
      titleKey: tool.titleKey,
      toolId: tool.id,
    },
  }));
}

export function initToolRoutes(router: Router) {
  for (const route of createToolRoutes()) {
    if (!router.hasRoute(route.name!)) {
      router.addRoute("RootLayout", route);
    }
  }
}

export function createToolMenuItems(t: (key: string) => string): MenuOption[] {
  return tools.map((tool) => ({
    label: t(tool.titleKey),
    key: tool.path,
    icon: tool.icon ? () => h(NIcon, { component: tool.icon }) : undefined,
  }));
}
