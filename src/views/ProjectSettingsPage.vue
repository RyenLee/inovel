<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useMessage } from "naive-ui";
import {
    NCard,
    NGrid,
    NGi,
    NButton,
    NForm,
    NFormItem,
    NInput,
    NSpace,
    NSpin,
    NIcon,
    NInputNumber,
    NTabs,
    NTabPane,
    NImage,
    NTag,
    NProgress,
} from "naive-ui";
import { ArrowLeft, Target, Save, FolderOpen, Keyboard, ImageIcon } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useProjectStore } from "../stores/project";
import ShortcutSettings from "../components/ShortcutSettings.vue";
import type { EncryptProjectParams, DecryptProjectParams, ChangePasswordParams, EncryptionProgress } from "../types/encryption";

const route = useRoute();
const router = useRouter();
const message = useMessage();
const projectStore = useProjectStore();

const projectId = computed(() => Number(route.params.projectId));
const isLoading = ref(false);
const isSaving = ref(false);
const activeTab = ref("basic");

// Project settings state
const projectData = ref({
    name: "",
    author: "",
    description: "",
    path: "",
});
const dailyGoal = ref(3000);
const coverUrl = ref<string | null>(null);
const isChangingCover = ref(false);

// 加密相关状态
const encryptPassword = ref("");
const encryptConfirmPassword = ref("");
const oldPassword = ref("");
const newPassword = ref("");
const confirmNewPassword = ref("");
const decryptPassword = ref("");
const isProcessing = ref(false);
const encryptProgress = ref<EncryptionProgress | null>(null);
const isEncrypted = ref(false);

// Get project info
const projectName = computed(() => {
    return projectStore.currentProject?.name || "项目";
});

onMounted(async () => {
    if (!projectStore.currentProject || projectStore.currentProject.id !== projectId.value) {
        await projectStore.openProject(projectId.value);
    }
    await loadSettings();
    
    // 检查项目是否已加密
    if (projectStore.currentProject) {
        isEncrypted.value = await projectStore.isProjectEncrypted(projectStore.currentProject.path);
    }
    
    // 监听加密进度
    const app = getCurrentWindow();
    app.listen('encryption-progress', (event: any) => {
        encryptProgress.value = event.payload;
    });
    
    // 监听解密进度
    app.listen('decryption-progress', (event: any) => {
        encryptProgress.value = event.payload;
    });
});

const loadSettings = async () => {
    isLoading.value = true;
    try {
        // Load project info
        if (projectStore.currentProject) {
            projectData.value = {
                name: projectStore.currentProject.name,
                author: projectStore.currentProject.author,
                description: projectStore.currentProject.description,
                path: projectStore.currentProject.path,
            };
        }

        // Load writing goal
        const goal = await invoke<{ daily_goal: number } | null>("get_writing_goal", {
            projectId: projectId.value,
        });
        if (goal) {
            dailyGoal.value = goal.daily_goal;
        }

        // Load cover
        if (projectStore.currentProject?.cover_path) {
            coverUrl.value = convertFileSrc(projectStore.currentProject.cover_path);
        }
    } catch (error) {
        console.error("Failed to load settings:", error);
    } finally {
        isLoading.value = false;
    }
};

// Change cover image
const changeCover = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: "Images",
                extensions: ["jpg", "jpeg", "png"],
            }],
        });

        if (!selected) return;

        isChangingCover.value = true;
        const newCoverPath = await invoke<string>("set_cover", {
            projectId: projectId.value,
            imagePath: selected,
        });

        coverUrl.value = convertFileSrc(newCoverPath);
        message.success("封面已更新");

        // Update current project
        if (projectStore.currentProject) {
            projectStore.currentProject.cover_path = newCoverPath;
        }
    } catch (error) {
        console.error("Failed to change cover:", error);
        message.error(`更换封面失败: ${error}`);
    } finally {
        isChangingCover.value = false;
    }
};

const saveProjectInfo = async () => {
    if (!projectData.value.name.trim()) {
        message.warning("请输入书名");
        return;
    }

    isSaving.value = true;
    try {
        const project = await projectStore.updateProject(projectId.value, {
            name: projectData.value.name.trim(),
            author: projectData.value.author.trim(),
            description: projectData.value.description.trim(),
        });

        if (project) {
            message.success("项目信息已保存");
        } else {
            message.error(projectStore.error || "保存失败");
        }
    } catch (error) {
        message.error(`保存失败: ${error}`);
    } finally {
        isSaving.value = false;
    }
};

const saveDailyGoal = async () => {
    isSaving.value = true;
    try {
        await invoke("save_writing_goal", {
            projectId: projectId.value,
            dailyGoal: dailyGoal.value,
        });
        message.success("每日目标已保存");
    } catch (error) {
        console.error("Failed to save daily goal:", error);
        message.error("保存失败");
    } finally {
        isSaving.value = false;
    }
};

const goBack = () => {
    router.push(`/editor/${projectId.value}`);
};

// 加密相关方法
const handleEncrypt = async () => {
    if (encryptPassword.value !== encryptConfirmPassword.value) {
        message.error("两次输入的密码不一致");
        return;
    }
    if (encryptPassword.value.length < 8) {
        message.error("密码长度至少 8 位");
        return;
    }
    
    isProcessing.value = true;
    try {
        const params = {
            project_path: projectStore.currentProject?.path || "",
            password: encryptPassword.value,
            confirm_password: encryptConfirmPassword.value,
        };
        await invoke("encrypt_project", { params });
        isEncrypted.value = true;
        encryptPassword.value = "";
        encryptConfirmPassword.value = "";
        message.success("项目已加密");
    } catch (e) {
        message.error(`加密失败: ${e}`);
    } finally {
        isProcessing.value = false;
    }
};

const handleChangePassword = async () => {
    if (newPassword.value !== confirmNewPassword.value) {
        message.error("两次输入的新密码不一致");
        return;
    }
    if (newPassword.value.length < 8) {
        message.error("新密码长度至少 8 位");
        return;
    }
    
    isProcessing.value = true;
    try {
        const params = {
            project_path: projectStore.currentProject?.path || "",
            old_password: oldPassword.value,
            new_password: newPassword.value,
            confirm_password: confirmNewPassword.value,
        };
        await invoke("change_project_password", { params });
        oldPassword.value = "";
        newPassword.value = "";
        confirmNewPassword.value = "";
        message.success("密码已修改");
    } catch (e) {
        message.error(`修改密码失败: ${e}`);
    } finally {
        isProcessing.value = false;
    }
};

const handleDecrypt = async () => {
    isProcessing.value = true;
    try {
        await invoke("disable_encryption", { 
            project_path: projectStore.currentProject?.path || "", 
            password: decryptPassword.value 
        });
        isEncrypted.value = false;
        decryptPassword.value = "";
        message.success("项目已解密，加密已关闭");
    } catch (e) {
        message.error(`关闭加密失败: ${e}`);
    } finally {
        isProcessing.value = false;
    }
};
</script>

<template>
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300">
        <header class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300">
            <div class="max-w-3xl mx-auto px-4 py-4 flex items-center gap-4">
                <n-button quaternary circle @click="goBack">
                    <template #icon>
                        <NIcon>
                            <ArrowLeft />
                        </NIcon>
                    </template>
                </n-button>
                <Target class="w-6 h-6 text-blue-600" />
                <h1 class="text-xl font-bold text-gray-900 dark:text-white">项目设置</h1>
                <span class="text-sm text-gray-500 dark:text-gray-400">{{ projectName }}</span>
            </div>
        </header>

        <main class="max-w-3xl mx-auto px-4 py-6">
            <!-- Tabs -->
            <n-tabs v-model:value="activeTab" type="line" class="mb-6">
                <n-tab-pane name="basic" tab="基本设置">
                    <div v-if="isLoading" class="flex justify-center py-12">
                        <n-spin size="large" />
                    </div>
                    <n-grid v-else :cols="1" :x-gap="16" :y-gap="16">
                        <!-- Cover Image -->
                        <n-gi>
                            <n-card title="封面图片" hoverable>
                                <div class="flex items-center gap-6">
                                    <div class="w-32 h-44 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 flex items-center justify-center">
                                        <img 
                                            v-if="coverUrl" 
                                            :src="coverUrl" 
                                            alt="项目封面"
                                            class="w-full h-full object-cover"
                                        />
                                        <n-icon size="48" class="text-gray-400" v-else>
                                            <ImageIcon />
                                        </n-icon>
                                    </div>
                                    <div class="flex flex-col gap-2">
                                        <n-button 
                                            type="primary" 
                                            @click="changeCover" 
                                            :loading="isChangingCover"
                                        >
                                            更换封面
                                        </n-button>
                                        <span class="text-sm text-gray-500 dark:text-gray-400">
                                            支持 JPG、PNG 格式
                                        </span>
                                    </div>
                                </div>
                            </n-card>
                        </n-gi>

                        <!-- Project Info -->
                        <n-gi>
                            <n-card title="项目信息" hoverable>
                                <n-form label-placement="top">
                                    <n-form-item label="项目ID">
                                        <n-input 
                                            :value="projectStore.currentProject?.project_id || ''" 
                                            readonly 
                                            placeholder="项目ID" 
                                        >
                                            <template #suffix>
                                                <span class="text-xs text-gray-400">不可修改</span>
                                            </template>
                                        </n-input>
                                    </n-form-item>
                                    <n-form-item label="书名">
                                        <n-input v-model:value="projectData.name" placeholder="请输入书名" maxlength="100" show-count />
                                    </n-form-item>
                                    <n-form-item label="笔名">
                                        <n-input v-model:value="projectData.author" placeholder="请输入作者笔名" maxlength="50" />
                                    </n-form-item>
                                    <n-form-item label="简介">
                                        <n-input v-model:value="projectData.description" type="textarea" placeholder="请输入小说简介" :rows="3" maxlength="500" show-count />
                                    </n-form-item>
                                    <n-form-item label="存储路径">
                                        <n-input v-model:value="projectData.path" readonly />
                                    </n-form-item>
                                </n-form>
                                <template #footer>
                                    <n-space justify="end">
                                        <n-button type="primary" @click="saveProjectInfo" :loading="isSaving">
                                            保存项目信息
                                        </n-button>
                                    </n-space>
                                </template>
                            </n-card>
                        </n-gi>

                        <!-- Writing Goals -->
                        <n-gi>
                            <n-card title="写作目标" hoverable>
                                <n-form label-placement="top">
                                    <n-form-item label="每日字数目标">
                                        <div class="flex items-center gap-4">
                                            <n-input-number
                                                v-model:value="dailyGoal"
                                                :min="0"
                                                :max="100000"
                                                :step="100"
                                                class="w-48"
                                            />
                                            <span class="text-gray-500 dark:text-gray-400">字/天</span>
                                        </div>
                                        <template #feedback>
                                            <span class="text-sm text-gray-500 dark:text-gray-400">
                                                设置此项目的每日写作目标，将覆盖全局设置
                                            </span>
                                        </template>
                                    </n-form-item>
                                </n-form>
                                <template #footer>
                                    <n-space justify="end">
                                        <n-button type="primary" @click="saveDailyGoal" :loading="isSaving">
                                            <template #icon>
                                                <NIcon><Save /></NIcon>
                                            </template>
                                            保存目标
                                        </n-button>
                                    </n-space>
                                </template>
                            </n-card>
                        </n-gi>
                    </n-grid>
                </n-tab-pane>

                <n-tab-pane name="shortcuts" tab="快捷键">
                    <ShortcutSettings :project-id="projectId" />
                </n-tab-pane>

                <n-tab-pane name="worldbuilding" tab="世界观">
                    <n-card title="世界观设定" hoverable>
                        <div class="flex items-center justify-center gap-2 text-sm text-gray-500 dark:text-gray-400 py-4">
                            <span>点击编辑器左侧的世界图标进入世界观设定面板</span>
                        </div>
                    </n-card>
                </n-tab-pane>

                <n-tab-pane name="security" tab="安全性">
                    <div v-if="isLoading" class="flex justify-center py-12">
                        <n-spin size="large" />
                    </div>
                    <n-grid v-else :cols="1" :x-gap="16" :y-gap="16">
                        <!-- 加密状态显示 -->
                        <n-gi>
                            <n-card title="加密状态" hoverable>
                                <div class="flex items-center gap-4">
                                    <n-tag :type="isEncrypted ? 'success' : 'default'">
                                        {{ isEncrypted ? '已加密' : '未加密' }}
                                    </n-tag>
                                    <span class="text-sm text-gray-500 dark:text-gray-400">
                                        {{ isEncrypted ? '项目文件已加密存储' : '项目文件未加密' }}
                                    </span>
                                </div>
                            </n-card>
                        </n-gi>

                        <!-- 设置密码/启用加密 -->
                        <n-gi v-if="!isEncrypted">
                            <n-card title="启用加密" hoverable>
                                <n-form label-placement="top">
                                    <n-form-item label="设置密码">
                                        <n-input 
                                            v-model:value="encryptPassword" 
                                            type="password" 
                                            placeholder="请输入密码（至少8位）"
                                            show-password-on="mousedown"
                                        />
                                    </n-form-item>
                                    <n-form-item label="确认密码">
                                        <n-input 
                                            v-model:value="encryptConfirmPassword" 
                                            type="password" 
                                            placeholder="请再次输入密码"
                                            show-password-on="mousedown"
                                        />
                                    </n-form-item>
                                    <n-form-item>
                                        <n-button 
                                            type="primary" 
                                            @click="handleEncrypt"
                                            :loading="isProcessing"
                                            :disabled="!encryptPassword || !encryptConfirmPassword"
                                        >
                                            启用加密
                                        </n-button>
                                    </n-form-item>
                                </n-form>
                                <n-progress 
                                    v-if="encryptProgress" 
                                    type="line" 
                                    :percentage="Math.round(encryptProgress.current / encryptProgress.total * 100)"
                                    :indicator-placement="'inside'"
                                    :processing="isProcessing"
                                />
                                <div v-if="encryptProgress" class="text-sm text-gray-500 mt-2">
                                    正在加密: {{ encryptProgress.current }} / {{ encryptProgress.total }} ({{ encryptProgress.currentFile }})
                                </div>
                            </n-card>
                        </n-gi>

                        <!-- 修改密码 -->
                        <n-gi v-if="isEncrypted">
                            <n-card title="修改密码" hoverable>
                                <n-form label-placement="top">
                                    <n-form-item label="原密码">
                                        <n-input 
                                            v-model:value="oldPassword" 
                                            type="password" 
                                            placeholder="请输入原密码"
                                            show-password-on="mousedown"
                                        />
                                    </n-form-item>
                                    <n-form-item label="新密码">
                                        <n-input 
                                            v-model:value="newPassword" 
                                            type="password" 
                                            placeholder="请输入新密码（至少8位）"
                                            show-password-on="mousedown"
                                        />
                                    </n-form-item>
                                    <n-form-item label="确认新密码">
                                        <n-input 
                                            v-model:value="confirmNewPassword" 
                                            type="password" 
                                            placeholder="请再次输入新密码"
                                            show-password-on="mousedown"
                                        />
                                    </n-form-item>
                                    <n-form-item>
                                        <n-button 
                                            type="warning" 
                                            @click="handleChangePassword"
                                            :loading="isProcessing"
                                            :disabled="!oldPassword || !newPassword || !confirmNewPassword"
                                        >
                                            修改密码
                                        </n-button>
                                    </n-form-item>
                                </n-form>
                            </n-card>
                        </n-gi>

                        <!-- 关闭加密 -->
                        <n-gi v-if="isEncrypted">
                            <n-card title="关闭加密" hoverable>
                                <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                                    关闭加密将解密所有项目文件。需要输入密码确认。
                                </p>
                                <n-form label-placement="top">
                                    <n-form-item label="密码">
                                        <n-input 
                                            v-model:value="decryptPassword" 
                                            type="password" 
                                            placeholder="请输入密码"
                                            show-password-on="mousedown"
                                        />
                                    </n-form-item>
                                    <n-form-item>
                                        <n-button 
                                            type="error" 
                                            @click="handleDecrypt"
                                            :loading="isProcessing"
                                            :disabled="!decryptPassword"
                                        >
                                            关闭加密
                                        </n-button>
                                    </n-form-item>
                                </n-form>
                            </n-card>
                        </n-gi>
                    </n-grid>
                </n-tab-pane>
            </n-tabs>
        </main>
    </div>
</template>
