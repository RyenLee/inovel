import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ProjectMeta {
  id: number;
  /** 项目唯一标识符（字母+数字组合，如 "P7K3M9"） */
  project_id: string;
  name: string;
  author: string;
  description: string;
  path: string;
  created_at: string;
  last_opened_at?: string;
  is_valid: boolean;
  cover_path?: string;
}

export interface CreateProjectParams {
  name: string;
  author: string;
  description: string;
  path: string;
}

export interface UpdateProjectParams {
  name: string;
  author: string;
  description: string;
}

export interface MigrationDetail {
  project_db_id: number;
  old_name: string;
  old_path: string;
  new_path: string;
  project_id: string;
  status: "success" | "skipped" | "failed" | "pending";
  error?: string;
}

export interface MigrateResult {
  total: number;
  success: number;
  failed: number;
  skipped: number;
  backup_path: string;
  details: MigrationDetail[];
}

export interface RollbackParams {
  project_ids?: number[];
}

export const useProjectStore = defineStore("project", () => {
  const recentProjects = ref<ProjectMeta[]>([]);
  const currentProject = ref<ProjectMeta | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function fetchRecentProjects() {
    isLoading.value = true;
    error.value = null;
    try {
      const projects = await invoke<ProjectMeta[]>("get_recent_projects");
      recentProjects.value = projects;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to fetch recent projects:", e);
    } finally {
      isLoading.value = false;
    }
  }

  async function openProject(id: number): Promise<{ success: boolean; project?: ProjectMeta }> {
    isLoading.value = true;
    error.value = null;
    try {
      const project = await invoke<ProjectMeta>("open_project", { id });

      // Check if project path is still valid
      if (!project.is_valid) {
        error.value = `项目路径失效：${project.path}`;
        return { success: false, project };
      }

      currentProject.value = project;

      // Update in recentProjects list
      const index = recentProjects.value.findIndex(p => p.id === id);
      if (index !== -1) {
        recentProjects.value[index] = project;
      }

      return { success: true, project };
    } catch (e) {
      error.value = String(e);
      console.error("Failed to open project:", e);
      return { success: false };
    } finally {
      isLoading.value = false;
    }
  }

  async function createProject(params: CreateProjectParams): Promise<ProjectMeta | null> {
    isLoading.value = true;
    error.value = null;
    try {
      const project = await invoke<ProjectMeta>("create_project", { params });
      recentProjects.value.unshift(project);
      currentProject.value = project;
      return project;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to create project:", e);
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  async function removeProjectFromList(id: number, keepFiles: boolean = true): Promise<boolean> {
    try {
      await invoke("remove_project_from_list", { id, keepFiles });
      recentProjects.value = recentProjects.value.filter(p => p.id !== id);
      return true;
    } catch (e) {
      console.error("Failed to remove project from list:", e);
      return false;
    }
  }

  async function updateProject(id: number, params: UpdateProjectParams): Promise<ProjectMeta | null> {
    isLoading.value = true;
    error.value = null;
    try {
      const project = await invoke<ProjectMeta>("update_project", { id, params });
      // Update in recentProjects list
      const index = recentProjects.value.findIndex(p => p.id === id);
      if (index !== -1) {
        recentProjects.value[index] = project;
      }
      if (currentProject.value?.id === id) {
        currentProject.value = project;
      }
      return project;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to update project:", e);
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  function setCurrentProject(project: ProjectMeta | null) {
    currentProject.value = project;
  }

  // === 数据迁移相关 ===

  /** 迁移状态 */
  const isMigrating = ref(false);
  const migrationResult = ref<MigrateResult | null>(null);

  /** 检查是否需要迁移，返回待迁移项目数 */
  async function checkMigrationNeeded(): Promise<number> {
    try {
      return await invoke<number>("check_migration_needed");
    } catch (e) {
      console.error("检查迁移需求失败:", e);
      return 0;
    }
  }

  /** 执行数据迁移 */
  async function migrateProjects(dryRun: boolean = false): Promise<MigrateResult | null> {
    isMigrating.value = true;
    error.value = null;
    try {
      const result = await invoke<MigrateResult>("migrate_existing_projects", { dryRun });
      migrationResult.value = result;
      return result;
    } catch (e) {
      error.value = String(e);
      console.error("迁移失败:", e);
      return null;
    } finally {
      isMigrating.value = false;
    }
  }

  /** 回滚迁移 */
  async function rollbackMigration(projectIds?: number[]): Promise<MigrateResult | null> {
    isMigrating.value = true;
    error.value = null;
    try {
      const params = projectIds ? { project_ids: projectIds } : undefined;
      const result = await invoke<MigrateResult>("rollback_migration", {
        params: params ? { project_ids: projectIds } : null,
      });
      migrationResult.value = result;
      return result;
    } catch (e) {
      error.value = String(e);
      console.error("回滚失败:", e);
      return null;
    } finally {
      isMigrating.value = false;
    }
  }

  // Keep getRecentProjects for backwards compatibility
  const getRecentProjects = fetchRecentProjects;

  return {
    recentProjects,
    currentProject,
    isLoading,
    error,
    fetchRecentProjects,
    getRecentProjects,
    openProject,
    createProject,
    removeProjectFromList,
    updateProject,
    setCurrentProject,

    // 迁移相关
    isMigrating,
    migrationResult,
    checkMigrationNeeded,
    migrateProjects,
    rollbackMigration,
  };
});
