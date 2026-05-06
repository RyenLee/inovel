<template>
  <div class="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
    <!-- Header -->
    <div
      class="flex items-center justify-between px-3 py-2.5 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex-nowrap gap-2">
      <div class="flex items-center gap-2 min-w-0 flex-1">
        <BookOpen class="w-4 h-4 text-blue-600 shrink-0" />
        <h3 class="font-semibold text-gray-900 dark:text-white whitespace-nowrap text-sm">世界观设定</h3>
        <span
          class="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap hidden sm:inline overflow-hidden text-ellipsis">
          ({{store.characters.length }}/{{ store.locations.length }}/{{ store.organizations.length }})</span>
      </div>
      <div class="shrink-0">
        <n-tabs v-model:value="activeTab" type="segment" size="small" class="shrink-0 worldbuilding-tabs">
          <n-tab-pane name="characters" tab="人物" />
          <n-tab-pane name="locations" tab="地点" />
          <n-tab-pane name="organizations" tab="组织" />
        </n-tabs>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <!-- Characters Content -->
      <div v-show="activeTab === 'characters'" class="p-4">
        <div class="flex justify-end mb-4">
          <n-button type="primary" size="small" @click="openCharacterDrawer()">
            <template #icon>
              <n-icon>
                <Plus />
              </n-icon>
            </template>
            新建人物
          </n-button>
        </div>

        <!-- Character Cards -->
        <div v-if="store.characters.length > 0" class="space-y-3">
          <div v-for="character in store.characters" :key="character.id"
            class="group relative bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-gray-700 hover:shadow-lg hover:border-blue-200 dark:hover:border-blue-700 cursor-pointer transition-all duration-300"
            :class="{ 'ring-2 ring-blue-500 ring-offset-2 dark:ring-offset-gray-900': false }"
            @click="viewCharacterDetail(character)">
            <!-- Avatar & Basic Info -->
            <div class="flex items-start gap-3">
              <!-- Avatar -->
              <div class="w-12 h-12 rounded-full flex items-center justify-center text-lg font-bold text-white shrink-0"
                :class="character.gender === 'male' ? 'bg-linear-to-br from-blue-500 to-blue-600' : character.gender === 'female' ? 'bg-linear-to-br from-pink-500 to-pink-600' : 'bg-linear-to-br from-gray-500 to-gray-600'">
                {{ character.name.charAt(0).toUpperCase() }}
              </div>

              <!-- Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h4 class="font-semibold text-base text-gray-900 dark:text-white truncate">{{ character.name }}</h4>
                  <n-button text type="error" size="tiny" class="opacity-0 group-hover:opacity-100 transition-opacity"
                    @click.stop="handleDeleteCharacter(character.id)">
                    <n-icon>
                      <Trash />
                    </n-icon>
                  </n-button>
                </div>

                <!-- Tags -->
                <div class="flex flex-wrap items-center gap-1.5 mt-1.5">
                  <n-tag v-if="character.gender" size="small" :type="character.gender === 'male' ? 'info' : 'warning'"
                    :bordered="false" class="rounded-full!">
                    {{ character.gender === 'male' ? '♂ 男' : '♀ 女' }}
                  </n-tag>
                  <n-tag v-if="character.age" size="small" type="default" :bordered="false" class="rounded-full!">
                    {{ character.age }}岁
                  </n-tag>
                </div>

                <!-- Description Preview -->
                <p v-if="character.personality"
                  class="text-sm text-gray-500 dark:text-gray-400 mt-2 line-clamp-2 leading-relaxed">
                  {{ character.personality }}
                </p>
              </div>
            </div>

            <!-- Hover indicator -->
            <div
              class="absolute inset-0 rounded-xl bg-linear-to-r from-blue-500/5 to-purple-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none" />
          </div>
        </div>

        <n-empty v-else description="暂无人物，点击上方按钮创建" class="mt-8" />
      </div>

      <!-- Locations Content -->
      <div v-show="activeTab === 'locations'" class="p-4">
        <div class="flex justify-end mb-4">
          <n-button type="primary" size="small" @click="openLocationDrawer()">
            <template #icon>
              <n-icon>
                <Plus />
              </n-icon>
            </template>
            新建地点
          </n-button>
        </div>

        <!-- Location Cards -->
        <div v-if="store.locations.length > 0" class="space-y-3">
          <div v-for="location in store.locations" :key="location.id"
            class="group relative bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-gray-700 hover:shadow-lg hover:border-green-200 dark:hover:border-green-700 cursor-pointer transition-all duration-300"
            @click="openLocationDrawer(location)">
            <div class="flex items-start gap-3">
              <!-- Location Icon -->
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center text-lg shrink-0 bg-linear-to-br from-green-400 to-green-600">
                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h4 class="font-semibold text-base text-gray-900 dark:text-white truncate">{{ location.name }}</h4>
                  <n-button text type="error" size="tiny" class="opacity-0 group-hover:opacity-100 transition-opacity"
                    @click.stop="handleDeleteLocation(location.id)">
                    <n-icon>
                      <Trash />
                    </n-icon>
                  </n-button>
                </div>

                <div class="flex flex-wrap items-center gap-1.5 mt-1.5">
                  <n-tag v-if="location.location_type" size="small" type="success" :bordered="false"
                    class="rounded-full!">
                    {{ getLocationTypeLabel(location.location_type) }}
                  </n-tag>
                  <n-tag v-if="location.population" size="small" type="default" :bordered="false" class="rounded-full!">
                    {{ location.population.toLocaleString() }}人
                  </n-tag>
                </div>

                <p v-if="location.description"
                  class="text-sm text-gray-500 dark:text-gray-400 mt-2 line-clamp-2 leading-relaxed">
                  {{ location.description }}
                </p>
              </div>
            </div>

            <div
              class="absolute inset-0 rounded-xl bg-linear-to-r from-green-500/5 to-teal-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none" />
          </div>
        </div>

        <n-empty v-else description="暂无地点，点击上方按钮创建" class="mt-8" />
      </div>

      <!-- Organizations Content -->
      <div v-show="activeTab === 'organizations'" class="p-4">
        <div class="flex justify-end mb-4">
          <n-button type="primary" size="small" @click="openOrganizationDrawer()">
            <template #icon>
              <n-icon>
                <Plus />
              </n-icon>
            </template>
            新建组织
          </n-button>
        </div>

        <!-- Organization Cards -->
        <div v-if="store.organizations.length > 0" class="space-y-3">
          <div v-for="org in store.organizations" :key="org.id"
            class="group relative bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-gray-700 hover:shadow-lg hover:border-amber-200 dark:hover:border-amber-700 cursor-pointer transition-all duration-300"
            @click="openOrganizationDrawer(org)">
            <div class="flex items-start gap-3">
              <!-- Org Icon -->
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center text-lg shrink-0 bg-linear-to-br from-amber-400 to-amber-600">
                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                </svg>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h4 class="font-semibold text-base text-gray-900 dark:text-white truncate">{{ org.name }}</h4>
                  <n-button text type="error" size="tiny" class="opacity-0 group-hover:opacity-100 transition-opacity"
                    @click.stop="handleDeleteOrganization(org.id)">
                    <n-icon>
                      <Trash />
                    </n-icon>
                  </n-button>
                </div>

                <div class="flex flex-wrap items-center gap-1.5 mt-1.5">
                  <n-tag v-if="org.org_type" size="small" type="warning" :bordered="false" class="rounded-full!">
                    {{ getOrgTypeLabel(org.org_type) }}
                  </n-tag>
                  <n-tag v-if="org.member_count" size="small" type="default" :bordered="false" class="rounded-full!">
                    {{ org.member_count }}人
                  </n-tag>
                </div>

                <p v-if="org.description"
                  class="text-sm text-gray-500 dark:text-gray-400 mt-2 line-clamp-2 leading-relaxed">
                  {{ org.description }}
                </p>
              </div>
            </div>

            <div
              class="absolute inset-0 rounded-xl bg-linear-to-r from-amber-500/5 to-orange-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none" />
          </div>
        </div>

        <n-empty v-else description="暂无组织，点击上方按钮创建" class="mt-8" />
      </div>
    </div>

    <!-- Character Drawer -->
    <n-drawer v-model:show="characterDrawerVisible" :width="560" placement="right" :trap-focus="true"
      :block-scroll="true">
      <n-drawer-content :title="editingCharacter ? '编辑人物' : '新建人物'" closable>
        <n-form label-placement="top">
          <n-form-item label="姓名" required>
            <n-input v-model:value="characterForm.name" placeholder="输入人物姓名" />
          </n-form-item>

          <n-form-item label="性别">
            <n-radio-group v-model:value="characterForm.gender" name="gender">
              <n-space>
                <n-radio value="male">男</n-radio>
                <n-radio value="female">女</n-radio>
                <n-radio value="">未知</n-radio>
              </n-space>
            </n-radio-group>
          </n-form-item>

          <n-form-item label="年龄">
            <n-input-number v-model:value="characterForm.age" :min="0" :max="1000" placeholder="输入年龄"
              style="width: 100%" />
          </n-form-item>

          <n-form-item label="外貌">
            <n-input v-model:value="characterForm.appearance" type="textarea" placeholder="描述人物外貌特征" :rows="3" />
          </n-form-item>

          <n-form-item label="性格">
            <n-input v-model:value="characterForm.personality" type="textarea" placeholder="描述人物性格特点" :rows="3" />
          </n-form-item>

          <n-form-item label="背景故事">
            <n-input v-model:value="characterForm.background" type="textarea" placeholder="描述人物背景故事" :rows="4" />
          </n-form-item>

          <n-divider>自定义属性</n-divider>

          <n-form-item label="添加自定义属性">
            <n-dynamic-input v-model:value="characterCustomFields" preset="pair" key-placeholder="属性名"
              value-placeholder="属性值" />
          </n-form-item>
        </n-form>

        <template #footer>
          <n-space justify="end">
            <n-button @click="characterDrawerVisible = false">取消</n-button>
            <n-button type="primary" @click="handleSaveCharacter">保存</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Location Drawer -->
    <n-drawer v-model:show="locationDrawerVisible" :width="560" placement="right" :trap-focus="true"
      :block-scroll="true">
      <n-drawer-content :title="editingLocation ? '编辑地点' : '新建地点'" closable>
        <n-form label-placement="top">
          <n-form-item label="名称" required>
            <n-input v-model:value="locationForm.name" placeholder="输入地点名称" />
          </n-form-item>

          <n-form-item label="类型">
            <n-select v-model:value="locationForm.location_type" :options="locationTypeOptions" placeholder="选择地点类型" />
          </n-form-item>

          <n-form-item label="描述">
            <n-input v-model:value="locationForm.description" type="textarea" placeholder="描述地点" :rows="3" />
          </n-form-item>

          <n-form-item label="气候">
            <n-input v-model:value="locationForm.climate" placeholder="描述当地气候" />
          </n-form-item>

          <n-form-item label="人口">
            <n-input-number v-model:value="locationForm.population" :min="0" placeholder="人口数量" style="width: 100%" />
          </n-form-item>

          <n-form-item label="特色">
            <n-input v-model:value="locationForm.notable_features" type="textarea" placeholder="描述地点的特色建筑或景观"
              :rows="3" />
          </n-form-item>

          <n-divider>自定义属性</n-divider>

          <n-form-item label="添加自定义属性">
            <n-dynamic-input v-model:value="locationCustomFields" preset="pair" key-placeholder="属性名"
              value-placeholder="属性值" />
          </n-form-item>
        </n-form>

        <template #footer>
          <n-space justify="end">
            <n-button @click="locationDrawerVisible = false">取消</n-button>
            <n-button type="primary" @click="handleSaveLocation">保存</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Organization Drawer -->
    <n-drawer v-model:show="organizationDrawerVisible" :width="560" placement="right" :trap-focus="true"
      :block-scroll="true">
      <n-drawer-content :title="editingOrganization ? '编辑组织' : '新建组织'" closable>
        <n-form label-placement="top">
          <n-form-item label="名称" required>
            <n-input v-model:value="organizationForm.name" placeholder="输入组织名称" />
          </n-form-item>

          <n-form-item label="类型">
            <n-select v-model:value="organizationForm.org_type" :options="orgTypeOptions" placeholder="选择组织类型" />
          </n-form-item>

          <n-form-item label="描述">
            <n-input v-model:value="organizationForm.description" type="textarea" placeholder="描述组织" :rows="3" />
          </n-form-item>

          <n-form-item label="领导者">
            <n-input v-model:value="organizationForm.leader" placeholder="组织领导者" />
          </n-form-item>

          <n-form-item label="总部">
            <n-input v-model:value="organizationForm.headquarters" placeholder="组织总部所在地" />
          </n-form-item>

          <n-form-item label="成员数量">
            <n-input-number v-model:value="organizationForm.member_count" :min="0" placeholder="成员数量"
              style="width: 100%" />
          </n-form-item>

          <n-divider>自定义属性</n-divider>

          <n-form-item label="添加自定义属性">
            <n-dynamic-input v-model:value="organizationCustomFields" preset="pair" key-placeholder="属性名"
              value-placeholder="属性值" />
          </n-form-item>
        </n-form>

        <template #footer>
          <n-space justify="end">
            <n-button @click="organizationDrawerVisible = false">取消</n-button>
            <n-button type="primary" @click="handleSaveOrganization">保存</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Character Detail Drawer (Read-only) -->
    <n-drawer v-model:show="characterDetailVisible" :width="480" placement="right" :trap-focus="true"
      :block-scroll="true">
      <n-drawer-content :title="viewingCharacter?.name || '人物详情'" closable>
        <div v-if="viewingCharacter" class="space-y-6">
          <!-- Avatar & Basic Info -->
          <div class="flex flex-col items-center text-center pb-4 border-b border-gray-100 dark:border-gray-700">
            <div class="w-20 h-20 rounded-full flex items-center justify-center text-2xl font-bold text-white mb-3"
              :class="viewingCharacter.gender === 'male' ? 'bg-linear-to-br from-blue-500 to-blue-600' : viewingCharacter.gender === 'female' ? 'bg-linear-to-br from-pink-500 to-pink-600' : 'bg-linear-to-br from-gray-500 to-gray-600'">
              {{ viewingCharacter.name.charAt(0).toUpperCase() }}
            </div>
            <div class="flex items-center gap-2">
              <n-tag v-if="viewingCharacter.gender" :type="viewingCharacter.gender === 'male' ? 'info' : 'warning'"
                size="small">
                {{ viewingCharacter.gender === 'male' ? '♂ 男' : '♀ 女' }}
              </n-tag>
              <n-tag v-if="viewingCharacter.age" type="default" size="small">
                {{ viewingCharacter.age }}岁
              </n-tag>
            </div>
          </div>

          <!-- Personality -->
          <div v-if="viewingCharacter.personality">
            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
              性格特点
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed pl-3.5">{{ viewingCharacter.personality }}</p>
          </div>

          <!-- Appearance -->
          <div v-if="viewingCharacter.appearance">
            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-pink-500"></span>
              外貌特征
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed pl-3.5">{{ viewingCharacter.appearance }}</p>
          </div>

          <!-- Background -->
          <div v-if="viewingCharacter.background">
            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-purple-500"></span>
              背景故事
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed whitespace-pre-wrap pl-3.5">{{
              viewingCharacter.background }}</p>
          </div>

          <!-- Custom Fields -->
          <div v-if="viewingCharacterCustomFields.length > 0">
            <h4 class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-amber-500"></span>
              自定义属性
            </h4>
            <div class="space-y-2 pl-3.5">
              <div v-for="(field, index) in viewingCharacterCustomFields" :key="index" class="flex items-start gap-2">
                <span class="text-gray-500 dark:text-gray-400 min-w-[60px]">{{ field.key }}:</span>
                <span class="text-gray-700 dark:text-gray-200">{{ field.value }}</span>
              </div>
            </div>
          </div>
        </div>

        <template #footer>
          <n-space justify="end">
            <n-button @click="characterDetailVisible = false">关闭</n-button>
            <n-button type="primary" @click="editCharacterFromDetail">编辑人物</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { NIcon } from 'naive-ui'
import { Plus, Trash, BookOpen } from 'lucide-vue-next'
import {
  useWorldbuildingStore,
  type Character,
  type Location,
  type Organization,
  type CustomField
} from '@/stores/worldbuilding'
import { useProjectStore } from '@/stores/project'

const store = useWorldbuildingStore()
const projectStore = useProjectStore()

// Get current project ID
const getProjectId = () => projectStore.currentProject?.id ?? null

const activeTab = ref('characters')

// Drawer states
const characterDrawerVisible = ref(false)
const locationDrawerVisible = ref(false)
const organizationDrawerVisible = ref(false)

// Editing states
const editingCharacter = ref<Character | null>(null)
const editingLocation = ref<Location | null>(null)
const editingOrganization = ref<Organization | null>(null)

// Character detail view states (read-only)
const characterDetailVisible = ref(false)
const viewingCharacter = ref<Character | null>(null)
const viewingCharacterCustomFields = ref<CustomField[]>([])

// Form states
const characterForm = reactive({
  name: '',
  gender: '',
  age: null as number | null,
  appearance: '',
  personality: '',
  background: '',
})

const characterCustomFields = ref<CustomField[]>([])

const locationForm = reactive({
  name: '',
  location_type: '',
  description: '',
  climate: '',
  population: null as number | null,
  notable_features: '',
})

const locationCustomFields = ref<CustomField[]>([])

const organizationForm = reactive({
  name: '',
  org_type: '',
  description: '',
  leader: '',
  headquarters: '',
  member_count: null as number | null,
})

const organizationCustomFields = ref<CustomField[]>([])

// Options
const locationTypeOptions = [
  { label: '城市', value: 'city' },
  { label: '城镇', value: 'town' },
  { label: '村庄', value: 'village' },
  { label: '建筑', value: 'building' },
  { label: '区域', value: 'region' },
  { label: '国家', value: 'country' },
  { label: '其他', value: 'other' },
]

const orgTypeOptions = [
  { label: '王国', value: 'kingdom' },
  { label: '公会', value: 'guild' },
  { label: '帮派', value: 'gang' },
  { label: '教派', value: 'cult' },
  { label: '商会', value: 'company' },
  { label: '军队', value: 'military' },
  { label: '秘密组织', value: 'secret_society' },
  { label: '家族', value: 'family' },
  { label: '其他', value: 'other' },
]

// Helper functions
function getLocationTypeLabel(type: string): string {
  const found = locationTypeOptions.find(opt => opt.value === type)
  return found ? found.label : type
}

function getOrgTypeLabel(type: string): string {
  const found = orgTypeOptions.find(opt => opt.value === type)
  return found ? found.label : type
}

function parseCustomFields(jsonStr: string): CustomField[] {
  try {
    const parsed = JSON.parse(jsonStr)
    return Object.entries(parsed).map(([key, value]) => ({ key, value: String(value) }))
  } catch {
    return []
  }
}

function stringifyCustomFields(fields: CustomField[]): string {
  const obj: Record<string, string> = {}
  fields.forEach(f => {
    if (f.key.trim()) {
      obj[f.key.trim()] = f.value
    }
  })
  return JSON.stringify(obj)
}

function resetCharacterForm() {
  editingCharacter.value = null
  characterForm.name = ''
  characterForm.gender = ''
  characterForm.age = null
  characterForm.appearance = ''
  characterForm.personality = ''
  characterForm.background = ''
  characterCustomFields.value = []
}

function resetLocationForm() {
  editingLocation.value = null
  locationForm.name = ''
  locationForm.location_type = ''
  locationForm.description = ''
  locationForm.climate = ''
  locationForm.population = null
  locationForm.notable_features = ''
  locationCustomFields.value = []
}

function resetOrganizationForm() {
  editingOrganization.value = null
  organizationForm.name = ''
  organizationForm.org_type = ''
  organizationForm.description = ''
  organizationForm.leader = ''
  organizationForm.headquarters = ''
  organizationForm.member_count = null
  organizationCustomFields.value = []
}

// Open drawer functions
function openCharacterDrawer(character?: Character) {
  if (character) {
    editingCharacter.value = character
    characterForm.name = character.name
    characterForm.gender = character.gender
    characterForm.age = character.age
    characterForm.appearance = character.appearance
    characterForm.personality = character.personality
    characterForm.background = character.background
    characterCustomFields.value = parseCustomFields(character.custom_fields)
  } else {
    resetCharacterForm()
  }
  characterDrawerVisible.value = true
}

// View character detail (read-only)
function viewCharacterDetail(character: Character) {
  viewingCharacter.value = character
  viewingCharacterCustomFields.value = parseCustomFields(character.custom_fields)
  characterDetailVisible.value = true
}

// Edit character from detail view
function editCharacterFromDetail() {
  if (viewingCharacter.value) {
    characterDetailVisible.value = false
    openCharacterDrawer(viewingCharacter.value)
  }
}

function openLocationDrawer(location?: Location) {
  if (location) {
    editingLocation.value = location
    locationForm.name = location.name
    locationForm.location_type = location.location_type
    locationForm.description = location.description
    locationForm.climate = location.climate
    locationForm.population = location.population
    locationForm.notable_features = location.notable_features
    locationCustomFields.value = parseCustomFields(location.custom_fields)
  } else {
    resetLocationForm()
  }
  locationDrawerVisible.value = true
}

function openOrganizationDrawer(org?: Organization) {
  if (org) {
    editingOrganization.value = org
    organizationForm.name = org.name
    organizationForm.org_type = org.org_type
    organizationForm.description = org.description
    organizationForm.leader = org.leader
    organizationForm.headquarters = org.headquarters
    organizationForm.member_count = org.member_count
    organizationCustomFields.value = parseCustomFields(org.custom_fields)
  } else {
    resetOrganizationForm()
  }
  organizationDrawerVisible.value = true
}

// Save functions
async function handleSaveCharacter() {
  if (!characterForm.name.trim()) {
    return
  }

  const projectId = getProjectId()
  if (!projectId) {
    alert('请先在编辑器中打开一个项目')
    return
  }

  const customFieldsStr = stringifyCustomFields(characterCustomFields.value)

  if (editingCharacter.value) {
    await store.updateCharacter(editingCharacter.value.id, {
      name: characterForm.name,
      gender: characterForm.gender,
      age: characterForm.age,
      appearance: characterForm.appearance,
      personality: characterForm.personality,
      background: characterForm.background,
      custom_fields: customFieldsStr,
    })
  } else {
    await store.createCharacter({
      project_id: projectId,
      name: characterForm.name,
      gender: characterForm.gender,
      age: characterForm.age,
      appearance: characterForm.appearance,
      personality: characterForm.personality,
      background: characterForm.background,
      custom_fields: customFieldsStr,
    })
  }

  characterDrawerVisible.value = false
}

async function handleSaveLocation() {
  if (!locationForm.name.trim()) {
    return
  }

  const projectId = getProjectId()
  if (!projectId) {
    alert('请先在编辑器中打开一个项目')
    return
  }

  const customFieldsStr = stringifyCustomFields(locationCustomFields.value)

  if (editingLocation.value) {
    await store.updateLocation(editingLocation.value.id, {
      name: locationForm.name,
      location_type: locationForm.location_type,
      description: locationForm.description,
      climate: locationForm.climate,
      population: locationForm.population,
      notable_features: locationForm.notable_features,
      custom_fields: customFieldsStr,
    })
  } else {
    await store.createLocation({
      project_id: projectId,
      name: locationForm.name,
      location_type: locationForm.location_type,
      description: locationForm.description,
      climate: locationForm.climate,
      population: locationForm.population,
      notable_features: locationForm.notable_features,
      custom_fields: customFieldsStr,
    })
  }

  locationDrawerVisible.value = false
}

async function handleSaveOrganization() {
  if (!organizationForm.name.trim()) {
    return
  }

  const projectId = getProjectId()
  if (!projectId) {
    alert('请先在编辑器中打开一个项目')
    return
  }

  const customFieldsStr = stringifyCustomFields(organizationCustomFields.value)

  if (editingOrganization.value) {
    await store.updateOrganization(editingOrganization.value.id, {
      name: organizationForm.name,
      org_type: organizationForm.org_type,
      description: organizationForm.description,
      leader: organizationForm.leader,
      headquarters: organizationForm.headquarters,
      member_count: organizationForm.member_count,
      custom_fields: customFieldsStr,
    })
  } else {
    await store.createOrganization({
      project_id: projectId,
      name: organizationForm.name,
      org_type: organizationForm.org_type,
      description: organizationForm.description,
      leader: organizationForm.leader,
      headquarters: organizationForm.headquarters,
      member_count: organizationForm.member_count,
      custom_fields: customFieldsStr,
    })
  }

  organizationDrawerVisible.value = false
}

// Delete functions
async function handleDeleteCharacter(id: number) {
  await store.deleteCharacter(id)
}

async function handleDeleteLocation(id: number) {
  await store.deleteLocation(id)
}

async function handleDeleteOrganization(id: number) {
  await store.deleteOrganization(id)
}

// Load data on mount
onMounted(async () => {
  const projectId = getProjectId()
  if (projectId) {
    await store.loadAll(projectId)
  }
})

// Expose methods to parent component
defineExpose({
  viewCharacterDetail,
  openCharacterDrawer,
})
</script>

<style scoped>
:deep(.n-tab-pane) {
  padding: 16px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

:deep(.n-tab-pane > div[class*="space-y-"])>*+* {
  margin-top: 12px;
}

/* 紧凑的worldbuilding-tabs样式 */
:deep(.worldbuilding-tabs) {
  min-width: fit-content;
}

:deep(.worldbuilding-tabs .n-tabs-tab) {
  padding-left: 10px !important;
  padding-right: 10px !important;
  padding-top: 4px !important;
  padding-bottom: 4px !important;
  font-size: 12px !important;
  font-weight: 500 !important;
  white-space: nowrap;
}

:deep(.worldbuilding-tabs .n-tabs-tab--active) {
  font-weight: 600 !important;
}

@media screen and (max-width: 640px) {
  :deep(.n-tab-pane) {
    padding: 12px;
  }

  :deep(.n-tab-pane > div[class*="space-y-"])>*+* {
    margin-top: 8px;
  }

  :deep(.worldbuilding-tabs .n-tabs-tab) {
    padding-left: 8px !important;
    padding-right: 8px !important;
    font-size: 11px !important;
  }
}

@media screen and (min-width: 641px) and (max-width: 1024px) {
  :deep(.n-tab-pane > div[class*="space-y-"])>*+* {
    margin-top: 10px;
  }
}
</style>
