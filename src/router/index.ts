import { createRouter, createWebHashHistory } from "vue-router";
import WelcomePage from "../views/WelcomePage.vue";
import EditorPage from "../views/EditorPage.vue";
import SettingsPage from "../views/SettingsPage.vue";
import ProjectSettingsPage from "../views/ProjectSettingsPage.vue";
import StatsDashboard from "../views/StatsDashboard.vue";
import ProjectStatsDashboard from "../views/ProjectStatsDashboard.vue";
import WorldbuildingPage from "../views/WorldbuildingPage.vue";
import TaskChecklistPage from "../views/TaskChecklistPage.vue";
import ConfigManagerPage from "../views/ConfigManagerPage.vue";

const router = createRouter({
    history: createWebHashHistory(),
    routes: [
        {
            path: "/",
            name: "Welcome",
            component: WelcomePage,
        },
        {
            path: "/editor/:projectId",
            name: "Editor",
            component: EditorPage,
        },
        // 全局设置与统计
        {
            path: "/settings",
            name: "Settings",
            component: SettingsPage,
        },
        {
            path: "/stats",
            name: "Stats",
            component: StatsDashboard,
        },
        // 单项目设置与统计
        {
            path: "/editor/:projectId/project-settings",
            name: "ProjectSettings",
            component: ProjectSettingsPage,
        },
        {
            path: "/editor/:projectId/project-stats",
            name: "ProjectStats",
            component: ProjectStatsDashboard,
        },
        // 世界观设定
        {
            path: "/editor/:projectId/worldbuilding",
            name: "Worldbuilding",
            component: WorldbuildingPage,
        },
        // 任务清单
        {
            path: "/tasks",
            name: "TaskChecklist",
            component: TaskChecklistPage,
        },
        // 配置管理
        {
            path: "/config",
            name: "ConfigManager",
            component: ConfigManagerPage,
        },
    ],
});

export default router;
