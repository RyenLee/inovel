<script setup lang="ts">
import { ref, computed, onMounted, h, markRaw } from "vue";
import {
  VueFlow,
  MarkerType,
  Position,
  Handle,
  NodeDragEvent,
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import {
  NButton,
  NIcon,
  NModal,
  NSelect,
  NPopconfirm,
  useMessage,
  NEmpty,
  NDropdown,
  NSpin,
} from "naive-ui";
import { Plus, Users } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../composables/useTheme";
import { useLocale } from "../i18n/composables/useLocale";
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";

const { isDark } = useTheme();
const { t } = useLocale();

// Types
interface Character {
  id: number;
  name: string;
  gender: string;
  age: number | null;
  appearance: string;
  personality: string;
  background: string;
}

interface Relationship {
  id: number;
  project_id: number;
  source_id: number;
  target_id: number;
  relation_type: string;
  created_at: string;
}

interface GraphNode {
  id: string;
  type?: string;
  position: { x: number; y: number };
  data: { label: string; character: Character };
  sourcePosition: Position;
  targetPosition: Position;
}

interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label: string;
  type: string;
  markerEnd: MarkerType;
  data?: { editable: boolean };
}

// Props
const props = defineProps<{
  projectId: number;
}>();

// Emits
const emit = defineEmits<{
  (e: "select-character", character: Character): void;
}>();

// State
const message = useMessage();
const nodes = ref<GraphNode[]>([]);
const edges = ref<GraphEdge[]>([]);
const characters = ref<Character[]>([]);
const isLoading = ref(true);

// Add relationship modal state
const showAddModal = ref(false);
const newRelationSource = ref<number | null>(null);
const newRelationTarget = ref<number | null>(null);
const newRelationType = ref("");

// Edit edge modal state
const showEditEdgeModal = ref(false);
const editingEdgeId = ref<string | null>(null);
const editingEdgeLabel = ref("");

// Context menu state
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuSource = ref<number | null>(null);

// Load data
const loadData = async () => {
  isLoading.value = true;
  try {
    // Load characters
    const chars = await invoke<Character[]>("list_characters", {
      project_id: props.projectId,
    });
    characters.value = chars;

    // Load relationships
    const rels = await invoke<Relationship[]>("get_relationships", {
      project_id: props.projectId,
    });

    // Load saved positions from localStorage
    const savedPositions = loadSavedPositions();

    // Convert to nodes
    nodes.value = chars.map((char, index) => {
      const savedPos = savedPositions[char.id];
      return {
        id: String(char.id),
        type: "custom",
        position: savedPos || {
          x: 100 + (index % 4) * 200,
          y: 100 + Math.floor(index / 4) * 150,
        },
        data: { label: char.name, character: char },
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      };
    });

    // Convert to edges
    edges.value = rels.map((rel) => ({
      id: String(rel.id),
      source: String(rel.source_id),
      target: String(rel.target_id),
      label: rel.relation_type,
      type: "default",
      markerEnd: MarkerType.ArrowClosed,
      data: { editable: true },
    }));
  } catch (error) {
    console.error("Failed to load data:", error);
    message.error(t("relationshipGraph.messages.loadFailed"));
  } finally {
    isLoading.value = false;
  }
};

// Save/load positions from localStorage
const positionStorageKey = computed(() => `graph-positions-${props.projectId}`);

function loadSavedPositions(): Record<number, { x: number; y: number }> {
  try {
    const saved = localStorage.getItem(positionStorageKey.value);
    return saved ? JSON.parse(saved) : {};
  } catch {
    return {};
  }
}

function savePositions() {
  const positions: Record<number, { x: number; y: number }> = {};
  nodes.value.forEach((node) => {
    positions[Number(node.id)] = node.position;
  });
  localStorage.setItem(positionStorageKey.value, JSON.stringify(positions));
}

// Handle node drag end
const onNodeDragStop = (_event: NodeDragEvent) => {
  savePositions();
};

// Handle node click
const onNodeClick = (event: any) => {
  if (event?.node?.data?.character) {
    emit("select-character", event.node.data.character);
  }
};

// Handle edge double click (edit)
const onEdgeDoubleClick = (event: any) => {
  if (event?.edge) {
    editingEdgeId.value = event.edge.id;
    editingEdgeLabel.value = event.edge.label;
    showEditEdgeModal.value = true;
  }
};

// Update edge label
const updateEdgeLabel = async () => {
  if (!editingEdgeId.value || !newRelationType.value) return;

  try {
    const relationshipId = Number(editingEdgeId.value);
    await invoke("update_relationship", {
      relationship_id: relationshipId,
      params: { relation_type: newRelationType.value },
    });

    // Update local edge
    const edgeIndex = edges.value.findIndex(
      (e) => e.id === editingEdgeId.value
    );
    if (edgeIndex !== -1) {
      edges.value[edgeIndex].label = newRelationType.value;
    }

    message.success(t("relationshipGraph.messages.relationUpdated"));
    showEditEdgeModal.value = false;
    editingEdgeId.value = null;
    newRelationType.value = "";
  } catch (error) {
    console.error("Failed to update relationship:", error);
    message.error(t("relationshipGraph.messages.updateFailed"));
  }
};

// Delete relationship
const deleteRelationship = async (relationshipId: number) => {
  try {
    await invoke("delete_relationship", { relationship_id: relationshipId });
    edges.value = edges.value.filter((e) => e.id !== String(relationshipId));
    message.success(t("relationshipGraph.messages.relationDeleted"));
  } catch (error) {
    console.error("Failed to delete relationship:", error);
    message.error(t("relationshipGraph.messages.deleteFailed"));
  }
};

// Show add relationship modal
const showAddRelationshipModal = (sourceId?: number) => {
  newRelationSource.value = sourceId || null;
  newRelationTarget.value = null;
  newRelationType.value = "";
  showAddModal.value = true;
};

// Add relationship
const addRelationship = async () => {
  if (
    !newRelationSource.value ||
    !newRelationTarget.value ||
    !newRelationType.value
  ) {
    message.warning(t("relationshipGraph.messages.fillAllFields"));
    return;
  }

  if (newRelationSource.value === newRelationTarget.value) {
    message.warning(t("relationshipGraph.messages.sameSourceTarget"));
    return;
  }

  try {
    const result = await invoke<Relationship>("create_relationship", {
      params: {
        project_id: props.projectId,
        source_id: newRelationSource.value,
        target_id: newRelationTarget.value,
        relation_type: newRelationType.value,
      },
    });

    edges.value.push({
      id: String(result.id),
      source: String(result.source_id),
      target: String(result.target_id),
      label: result.relation_type,
      type: "default",
      markerEnd: MarkerType.ArrowClosed,
      data: { editable: true },
    });

    message.success(t("relationshipGraph.messages.relationAdded"));
    showAddModal.value = false;
  } catch (error) {
    console.error("Failed to create relationship:", error);
    message.error(t("relationshipGraph.messages.createFailed"));
  }
};

// Context menu
const onNodeContextMenu = (event: any) => {
  if (event?.node && event?.originalEvent) {
    event.originalEvent.preventDefault();
    contextMenuSource.value = Number(event.node.id);
    contextMenuX.value = event.originalEvent.clientX;
    contextMenuY.value = event.originalEvent.clientY;
    contextMenuVisible.value = true;
  }
};

const hideContextMenu = () => {
  contextMenuVisible.value = false;
};

const contextMenuOptions = computed(() => [
  {
    label: t("relationshipGraph.contextMenu.addFromSource"),
    key: "add-from-source",
  },
  {
    label: t("relationshipGraph.contextMenu.viewDetails"),
    key: "view-details",
  },
]);

const handleContextMenuSelect = (key: string) => {
  hideContextMenu();
  if (contextMenuSource.value) {
    if (key === "add-from-source") {
      showAddRelationshipModal(contextMenuSource.value);
    } else if (key === "view-details") {
      const char = characters.value.find(
        (c) => c.id === contextMenuSource.value
      );
      if (char) {
        emit("select-character", char);
      }
    }
  }
};

// Relation type options
const relationTypeOptions = [
  { label: t("relationshipGraph.relationTypes.friend"), value: "朋友" },
  { label: t("relationshipGraph.relationTypes.enemy"), value: "敌人" },
  { label: t("relationshipGraph.relationTypes.lover"), value: "恋人" },
  { label: t("relationshipGraph.relationTypes.family"), value: "家人" },
  { label: t("relationshipGraph.relationTypes.colleague"), value: "同事" },
  { label: t("relationshipGraph.relationTypes.superior"), value: "上下级" },
  { label: t("relationshipGraph.relationTypes.teacher"), value: "师生" },
  { label: t("relationshipGraph.relationTypes.brother"), value: "兄弟" },
  { label: t("relationshipGraph.relationTypes.sister"), value: "姐妹" },
  { label: t("relationshipGraph.relationTypes.rival"), value: "竞争对手" },
  { label: t("relationshipGraph.relationTypes.partner"), value: "合作伙伴" },
  { label: t("relationshipGraph.relationTypes.other"), value: "其他" },
];

// Character select options
const characterSelectOptions = computed(() =>
  characters.value.map((c) => ({
    label: c.name,
    value: c.id,
  }))
);

// Custom node component using Vue's h() render function
const CharacterNode = {
  props: ["data", "selected"],
  render(props: any) {
    const selectedClass = props.selected
      ? "border-blue-500 bg-blue-50 shadow-lg"
      : "border-gray-300 bg-white hover:border-blue-400";
    return h(
      "div",
      {
        class: `px-4 py-3 rounded-lg border-2 min-w-[120px] text-center transition-all ${selectedClass}`,
      },
      [
        h(Handle, {
          type: "target",
          position: Position.Left,
          class: "!bg-gray-400",
        }),
        h("div", { class: "font-medium text-gray-800" }, props.data.label),
        h(Handle, {
          type: "source",
          position: Position.Right,
          class: "!bg-gray-400",
        }),
      ]
    );
  },
};

// Create custom node types object
const customNodeTypes = {
  custom: markRaw(CharacterNode),
};

onMounted(() => {
  loadData();
  document.addEventListener("click", hideContextMenu);
});
</script>

<template>
  <div class="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
    <!-- Header -->
    <div
      class="flex items-center justify-between px-4 py-3 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700"
    >
      <div class="flex items-center gap-2">
        <Users class="w-5 h-5 text-blue-600" />
        <h3 class="font-semibold text-gray-900 dark:text-white">
          {{ t("relationshipGraph.title") }}
        </h3>
        <span class="text-sm text-gray-500 dark:text-gray-400"
          >({{
            t("relationshipGraph.stats", {
              charCount: characters.length,
              edgeCount: edges.length,
            })
          }})</span
        >
      </div>
      <NButton type="primary" size="small" @click="showAddRelationshipModal()">
        <template #icon>
          <NIcon><Plus /></NIcon>
        </template>
        {{ t("relationshipGraph.addRelation") }}
      </NButton>
    </div>

    <!-- Graph -->
    <div class="flex-1 relative">
      <div
        v-if="isLoading"
        class="absolute inset-0 flex items-center justify-center"
      >
        <NSpin size="large" />
      </div>

      <div
        v-else-if="characters.length === 0"
        class="absolute inset-0 flex items-center justify-center"
      >
        <NEmpty :description="t('relationshipGraph.noCharacters')">
          <template #extra>
            <p class="text-sm text-gray-500">
              {{ t("relationshipGraph.noCharactersHint") }}
            </p>
          </template>
        </NEmpty>
      </div>

      <VueFlow
        v-else
        v-model:nodes="nodes"
        v-model:edges="edges"
        :node-types="customNodeTypes"
        :nodes-default-position="[100, 100]"
        :default-viewport="{ zoom: 1 }"
        fit-view-on-init
        @node-drag-stop="onNodeDragStop"
        @node-click="onNodeClick"
        @node-context-menu="onNodeContextMenu"
        @edge-double-click="onEdgeDoubleClick"
        class="bg-gray-50 dark:bg-gray-900"
      >
        <Background :pattern-color="isDark ? '#555' : '#aaa'" :gap="16" />
        <Controls />
      </VueFlow>

      <!-- Context Menu -->
      <NDropdown
        v-model:show="contextMenuVisible"
        trigger="manual"
        :x="contextMenuX"
        :y="contextMenuY"
        :options="contextMenuOptions"
        @select="handleContextMenuSelect"
      />

      <!-- Add Relationship Modal -->
      <NModal
        v-model:show="showAddModal"
        preset="card"
        :title="t('relationshipGraph.addRelationTitle')"
        style="width: 400px"
      >
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-2">{{
              t("relationshipGraph.sourceCharacter")
            }}</label>
            <NSelect
              v-model:value="newRelationSource"
              :options="characterSelectOptions"
              :placeholder="t('relationshipGraph.selectSource')"
              filterable
            />
          </div>
          <div>
            <label class="block text-sm font-medium mb-2">{{
              t("relationshipGraph.targetCharacter")
            }}</label>
            <NSelect
              v-model:value="newRelationTarget"
              :options="
                characterSelectOptions.filter(
                  (o) => o.value !== newRelationSource
                )
              "
              :placeholder="t('relationshipGraph.selectTarget')"
              filterable
            />
          </div>
          <div>
            <label class="block text-sm font-medium mb-2">{{
              t("relationshipGraph.relationType")
            }}</label>
            <NSelect
              v-model:value="newRelationType"
              :options="relationTypeOptions"
              :placeholder="t('relationshipGraph.selectOrInputType')"
              filterable
              allow-create
            />
          </div>
        </div>
        <template #footer>
          <div class="flex justify-end gap-2">
            <NButton @click="showAddModal = false">{{
              t("relationshipGraph.cancel")
            }}</NButton>
            <NButton type="primary" @click="addRelationship">{{
              t("relationshipGraph.confirm")
            }}</NButton>
          </div>
        </template>
      </NModal>

      <!-- Edit Edge Modal -->
      <NModal
        v-model:show="showEditEdgeModal"
        preset="card"
        :title="t('relationshipGraph.editRelationTitle')"
        style="width: 400px"
      >
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-2">{{
              t("relationshipGraph.relationType")
            }}</label>
            <NSelect
              v-model:value="newRelationType"
              :options="relationTypeOptions"
              :placeholder="t('relationshipGraph.selectOrInputType')"
              filterable
              allow-create
            />
          </div>
        </div>
        <template #footer>
          <div class="flex justify-between">
            <NPopconfirm
              v-if="editingEdgeId"
              @positive-click="() => deleteRelationship(Number(editingEdgeId))"
            >
              <template #trigger>
                <NButton type="error" secondary>{{
                  t("relationshipGraph.deleteRelation")
                }}</NButton>
              </template>
              {{ t("relationshipGraph.deleteConfirm") }}
            </NPopconfirm>
            <div class="flex gap-2 ml-auto">
              <NButton @click="showEditEdgeModal = false">{{
                t("relationshipGraph.cancel")
              }}</NButton>
              <NButton type="primary" @click="updateEdgeLabel">{{
                t("relationshipGraph.save")
              }}</NButton>
            </div>
          </div>
        </template>
      </NModal>
    </div>
  </div>
</template>

<style scoped>
.vue-flow {
  height: 100%;
}
</style>
