import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { WritingTemplate, UserTemplate } from "../types/template";

export const useTemplateStore = defineStore("template", () => {
  // State
  const builtinTemplates = ref<WritingTemplate[]>([]);
  const userTemplates = ref<UserTemplate[]>([]);
  const isLoading = ref(false);

  // Getters
  const allTemplates = computed(() => {
    return [
      ...builtinTemplates.value,
      ...userTemplates.value.map(t => ({
        id: `user_${t.id}`,
        name: t.name,
        description: t.description,
        category: t.category,
        content: t.content,
        is_builtin: false,
      })),
    ];
  });

  // Actions
  async function loadBuiltinTemplates() {
    if (builtinTemplates.value.length > 0) return;

    isLoading.value = true;
    try {
      const templates = await invoke<WritingTemplate[]>("get_builtin_templates");
      builtinTemplates.value = templates;
    } catch (error) {
      console.error("加载内置模板失败:", error);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadUserTemplates(projectId: number) {
    isLoading.value = true;
    try {
      const templates = await invoke<UserTemplate[]>("get_user_templates", {
        projectId,
      });
      userTemplates.value = templates;
    } catch (error) {
      console.error("加载用户模板失败:", error);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadAllTemplates(projectId: number) {
    isLoading.value = true;
    try {
      const [builtin, user] = await invoke<[WritingTemplate[], UserTemplate[]]>(
        "get_all_templates",
        { projectId }
      );
      builtinTemplates.value = builtin;
      userTemplates.value = user;
    } catch (error) {
      console.error("加载模板失败:", error);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveUserTemplate(params: {
    project_id: number;
    name: string;
    description: string;
    category: string;
    content: string;
  }) {
    try {
      const template = await invoke<UserTemplate>("save_user_template", {
        params,
      });
      userTemplates.value.unshift(template);
      return template;
    } catch (error) {
      console.error("保存模板失败:", error);
      throw error;
    }
  }

  async function updateUserTemplate(
    templateId: number,
    params: {
      name?: string;
      description?: string;
      category?: string;
      content?: string;
    }
  ) {
    try {
      const template = await invoke<UserTemplate>("update_user_template", {
        templateId,
        params,
      });
      const index = userTemplates.value.findIndex(t => t.id === templateId);
      if (index !== -1) {
        userTemplates.value[index] = template;
      }
      return template;
    } catch (error) {
      console.error("更新模板失败:", error);
      throw error;
    }
  }

  async function deleteUserTemplate(templateId: number) {
    try {
      await invoke("delete_user_template", {
        templateId,
      });
      userTemplates.value = userTemplates.value.filter(t => t.id !== templateId);
    } catch (error) {
      console.error("删除模板失败:", error);
      throw error;
    }
  }

  return {
    builtinTemplates,
    userTemplates,
    allTemplates,
    isLoading,
    loadBuiltinTemplates,
    loadUserTemplates,
    loadAllTemplates,
    saveUserTemplate,
    updateUserTemplate,
    deleteUserTemplate,
  };
});
