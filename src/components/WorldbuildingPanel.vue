<template>
  <div class="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
    <!-- Header -->
    <div
      class="flex items-center justify-between px-3 py-2.5 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex-nowrap gap-2"
    >
      <div class="flex items-center gap-2 min-w-0 flex-1">
        <BookOpen class="w-4 h-4 text-blue-600 shrink-0" />
        <h3
          class="font-semibold text-gray-900 dark:text-white whitespace-nowrap text-sm"
        >
          {{ t("worldbuilding.title") }}
        </h3>
        <span
          class="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap hidden sm:inline overflow-hidden text-ellipsis"
        >
          ({{ store.characters.length }}/{{ store.locations.length }}/{{
            store.organizations.length
          }})</span
        >
      </div>
      <div class="shrink-0">
        <n-tabs
          v-model:value="activeTab"
          type="segment"
          size="small"
          class="shrink-0 worldbuilding-tabs"
        >
          <n-tab-pane name="characters" :tab="t('worldbuilding.characters')" />
          <n-tab-pane name="locations" :tab="t('worldbuilding.locations')" />
          <n-tab-pane
            name="organizations"
            :tab="t('worldbuilding.organizations')"
          />
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
            {{ t("worldbuilding.character.add") }}
          </n-button>
        </div>

        <!-- Character Cards -->
        <div v-if="store.characters.length > 0" class="space-y-3">
          <div
            v-for="character in store.characters"
            :key="character.id"
            class="group relative bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-gray-700 hover:shadow-lg hover:border-blue-200 dark:hover:border-blue-700 cursor-pointer transition-all duration-300"
            :class="{
              'ring-2 ring-blue-500 ring-offset-2 dark:ring-offset-gray-900': false,
            }"
            @click="viewCharacterDetail(character)"
          >
            <!-- Avatar & Basic Info -->
            <div class="flex items-start gap-3">
              <!-- Avatar -->
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center text-lg font-bold text-white shrink-0"
                :class="
                  character.gender === 'male'
                    ? 'bg-linear-to-br from-blue-500 to-blue-600'
                    : character.gender === 'female'
                    ? 'bg-linear-to-br from-pink-500 to-pink-600'
                    : 'bg-linear-to-br from-gray-500 to-gray-600'
                "
              >
                {{ character.name.charAt(0).toUpperCase() }}
              </div>

              <!-- Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h4
                    class="font-semibold text-base text-gray-900 dark:text-white truncate"
                  >
                    {{ character.name }}
                  </h4>
                  <n-button
                    text
                    type="error"
                    size="tiny"
                    class="opacity-0 group-hover:opacity-100 transition-opacity"
                    @click.stop="handleDeleteCharacter(character.id)"
                  >
                    <n-icon>
                      <Trash />
                    </n-icon>
                  </n-button>
                </div>

                <!-- Tags -->
                <div class="flex flex-wrap items-center gap-1.5 mt-1.5">
                  <n-tag
                    v-if="character.gender"
                    size="small"
                    :type="character.gender === 'male' ? 'info' : 'warning'"
                    :bordered="false"
                    class="rounded-full!"
                  >
                    {{ enumDictionary.getGenderName(character.gender) }}
                  </n-tag>
                  <n-tag
                    v-if="character.age"
                    size="small"
                    type="default"
                    :bordered="false"
                    class="rounded-full!"
                  >
                    {{ character.age }}{{ t("worldbuilding.ageUnit") }}
                  </n-tag>
                </div>

                <!-- Description Preview -->
                <p
                  v-if="character.personality"
                  class="text-sm text-gray-500 dark:text-gray-400 mt-2 line-clamp-2 leading-relaxed"
                >
                  {{ character.personality }}
                </p>
              </div>
            </div>

            <!-- Hover indicator -->
            <div
              class="absolute inset-0 rounded-xl bg-linear-to-r from-blue-500/5 to-purple-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"
            />
          </div>
        </div>

        <n-empty
          v-else
          :description="t('worldbuilding.character.empty')"
          class="mt-8"
        />
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
            {{ t("worldbuilding.location.add") }}
          </n-button>
        </div>

        <!-- Location Cards -->
        <div v-if="store.locations.length > 0" class="space-y-3">
          <div
            v-for="location in store.locations"
            :key="location.id"
            class="group relative bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-gray-700 hover:shadow-lg hover:border-green-200 dark:hover:border-green-700 cursor-pointer transition-all duration-300"
            @click="openLocationDrawer(location)"
          >
            <div class="flex items-start gap-3">
              <!-- Location Icon -->
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center text-lg shrink-0 bg-linear-to-br from-green-400 to-green-600"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                </svg>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h4
                    class="font-semibold text-base text-gray-900 dark:text-white truncate"
                  >
                    {{ location.name }}
                  </h4>
                  <n-button
                    text
                    type="error"
                    size="tiny"
                    class="opacity-0 group-hover:opacity-100 transition-opacity"
                    @click.stop="handleDeleteLocation(location.id)"
                  >
                    <n-icon>
                      <Trash />
                    </n-icon>
                  </n-button>
                </div>

                <div class="flex flex-wrap items-center gap-1.5 mt-1.5">
                  <n-tag
                    v-if="location.location_type"
                    size="small"
                    type="success"
                    :bordered="false"
                    class="rounded-full!"
                  >
                    {{ getLocationTypeLabel(location.location_type) }}
                  </n-tag>
                  <n-tag
                    v-if="location.population"
                    size="small"
                    type="default"
                    :bordered="false"
                    class="rounded-full!"
                  >
                    {{ location.population.toLocaleString()
                    }}{{ t("worldbuilding.peopleUnit") }}
                  </n-tag>
                </div>

                <p
                  v-if="location.description"
                  class="text-sm text-gray-500 dark:text-gray-400 mt-2 line-clamp-2 leading-relaxed"
                >
                  {{ location.description }}
                </p>
              </div>
            </div>

            <div
              class="absolute inset-0 rounded-xl bg-linear-to-r from-green-500/5 to-teal-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"
            />
          </div>
        </div>

        <n-empty
          v-else
          :description="t('worldbuilding.location.empty')"
          class="mt-8"
        />
      </div>

      <!-- Organizations Content -->
      <div v-show="activeTab === 'organizations'" class="p-4">
        <div class="flex justify-end mb-4">
          <n-button
            type="primary"
            size="small"
            @click="openOrganizationDrawer()"
          >
            <template #icon>
              <n-icon>
                <Plus />
              </n-icon>
            </template>
            {{ t("worldbuilding.organization.add") }}
          </n-button>
        </div>

        <!-- Organization Cards -->
        <div v-if="store.organizations.length > 0" class="space-y-3">
          <div
            v-for="org in store.organizations"
            :key="org.id"
            class="group relative bg-white dark:bg-gray-800 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-gray-700 hover:shadow-lg hover:border-amber-200 dark:hover:border-amber-700 cursor-pointer transition-all duration-300"
            @click="openOrganizationDrawer(org)"
          >
            <div class="flex items-start gap-3">
              <!-- Org Icon -->
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center text-lg shrink-0 bg-linear-to-br from-amber-400 to-amber-600"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"
                  />
                </svg>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <h4
                    class="font-semibold text-base text-gray-900 dark:text-white truncate"
                  >
                    {{ org.name }}
                  </h4>
                  <n-button
                    text
                    type="error"
                    size="tiny"
                    class="opacity-0 group-hover:opacity-100 transition-opacity"
                    @click.stop="handleDeleteOrganization(org.id)"
                  >
                    <n-icon>
                      <Trash />
                    </n-icon>
                  </n-button>
                </div>

                <div class="flex flex-wrap items-center gap-1.5 mt-1.5">
                  <n-tag
                    v-if="org.org_type"
                    size="small"
                    type="warning"
                    :bordered="false"
                    class="rounded-full!"
                  >
                    {{ getOrgTypeLabel(org.org_type) }}
                  </n-tag>
                  <n-tag
                    v-if="org.member_count"
                    size="small"
                    type="default"
                    :bordered="false"
                    class="rounded-full!"
                  >
                    {{ org.member_count }}{{ t("worldbuilding.peopleUnit") }}
                  </n-tag>
                </div>

                <p
                  v-if="org.description"
                  class="text-sm text-gray-500 dark:text-gray-400 mt-2 line-clamp-2 leading-relaxed"
                >
                  {{ org.description }}
                </p>
              </div>
            </div>

            <div
              class="absolute inset-0 rounded-xl bg-linear-to-r from-amber-500/5 to-orange-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"
            />
          </div>
        </div>

        <n-empty
          v-else
          :description="t('worldbuilding.organization.empty')"
          class="mt-8"
        />
      </div>
    </div>

    <!-- Character Drawer -->
    <n-drawer
      v-model:show="characterDrawerVisible"
      :width="560"
      placement="right"
      :trap-focus="true"
      :block-scroll="true"
    >
      <n-drawer-content
        :title="
          editingCharacter
            ? t('worldbuilding.character.edit')
            : t('worldbuilding.character.add')
        "
        closable
      >
        <n-form label-placement="top">
          <n-form-item :label="t('worldbuilding.character.name')" required>
            <n-input
              v-model:value="characterForm.name"
              :placeholder="t('worldbuilding.character.placeholder.name')"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.character.gender')">
            <n-select
              v-model:value="characterForm.gender"
              :options="enumDictionary.genderOptions.value"
              :placeholder="t('worldbuilding.character.placeholder.gender')"
              style="width: 100%"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.character.age')">
            <n-input-number
              v-model:value="characterForm.age"
              :min="0"
              :max="1000"
              :placeholder="t('worldbuilding.character.placeholder.age')"
              style="width: 100%"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.character.appearance')">
            <n-input
              v-model:value="characterForm.appearance"
              type="textarea"
              :placeholder="t('worldbuilding.character.placeholder.appearance')"
              :rows="3"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.character.personality')">
            <n-input
              v-model:value="characterForm.personality"
              type="textarea"
              :placeholder="
                t('worldbuilding.character.placeholder.personality')
              "
              :rows="3"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.character.background')">
            <n-input
              v-model:value="characterForm.background"
              type="textarea"
              :placeholder="t('worldbuilding.character.placeholder.background')"
              :rows="4"
            />
          </n-form-item>

          <n-divider>{{ t("worldbuilding.customFields") }}</n-divider>

          <n-form-item :label="t('worldbuilding.addCustomField')">
            <n-dynamic-input
              v-model:value="characterCustomFields"
              preset="pair"
              :key-placeholder="t('worldbuilding.fieldName')"
              :value-placeholder="t('worldbuilding.fieldValue')"
            />
          </n-form-item>
        </n-form>

        <template #footer>
          <n-space justify="end">
            <n-button @click="characterDrawerVisible = false">{{
              t("worldbuilding.cancel")
            }}</n-button>
            <n-button type="primary" @click="handleSaveCharacter">{{
              t("worldbuilding.save")
            }}</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Location Drawer -->
    <n-drawer
      v-model:show="locationDrawerVisible"
      :width="560"
      placement="right"
      :trap-focus="true"
      :block-scroll="true"
    >
      <n-drawer-content
        :title="
          editingLocation
            ? t('worldbuilding.location.edit')
            : t('worldbuilding.location.add')
        "
        closable
      >
        <n-form label-placement="top">
          <n-form-item :label="t('worldbuilding.location.name')" required>
            <n-input
              v-model:value="locationForm.name"
              :placeholder="t('worldbuilding.location.placeholder.name')"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.location.type')">
            <n-select
              v-model:value="locationForm.location_type"
              :options="enumDictionary.locationTypeOptions.value"
              :placeholder="t('worldbuilding.location.placeholder.type')"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.location.description')">
            <n-input
              v-model:value="locationForm.description"
              type="textarea"
              :placeholder="t('worldbuilding.location.placeholder.description')"
              :rows="3"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.location.climate')">
            <n-input
              v-model:value="locationForm.climate"
              :placeholder="t('worldbuilding.location.placeholder.climate')"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.location.population')">
            <n-input-number
              v-model:value="locationForm.population"
              :min="0"
              :placeholder="t('worldbuilding.location.placeholder.population')"
              style="width: 100%"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.location.notableFeatures')">
            <n-input
              v-model:value="locationForm.notable_features"
              type="textarea"
              :placeholder="
                t('worldbuilding.location.placeholder.notableFeatures')
              "
              :rows="3"
            />
          </n-form-item>

          <n-divider>{{ t("worldbuilding.customFields") }}</n-divider>

          <n-form-item :label="t('worldbuilding.addCustomField')">
            <n-dynamic-input
              v-model:value="locationCustomFields"
              preset="pair"
              :key-placeholder="t('worldbuilding.fieldName')"
              :value-placeholder="t('worldbuilding.fieldValue')"
            />
          </n-form-item>
        </n-form>

        <template #footer>
          <n-space justify="end">
            <n-button @click="locationDrawerVisible = false">{{
              t("worldbuilding.cancel")
            }}</n-button>
            <n-button type="primary" @click="handleSaveLocation">{{
              t("worldbuilding.save")
            }}</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Organization Drawer -->
    <n-drawer
      v-model:show="organizationDrawerVisible"
      :width="560"
      placement="right"
      :trap-focus="true"
      :block-scroll="true"
    >
      <n-drawer-content
        :title="
          editingOrganization
            ? t('worldbuilding.organization.edit')
            : t('worldbuilding.organization.add')
        "
        closable
      >
        <n-form label-placement="top">
          <n-form-item :label="t('worldbuilding.organization.name')" required>
            <n-input
              v-model:value="organizationForm.name"
              :placeholder="t('worldbuilding.organization.placeholder.name')"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.organization.type')">
            <n-select
              v-model:value="organizationForm.org_type"
              :options="enumDictionary.organizationTypeOptions.value"
              :placeholder="t('worldbuilding.organization.placeholder.type')"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.organization.description')">
            <n-input
              v-model:value="organizationForm.description"
              type="textarea"
              :placeholder="
                t('worldbuilding.organization.placeholder.description')
              "
              :rows="3"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.organization.leader')">
            <n-input
              v-model:value="organizationForm.leader"
              :placeholder="t('worldbuilding.organization.placeholder.leader')"
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.organization.headquarters')">
            <n-input
              v-model:value="organizationForm.headquarters"
              :placeholder="
                t('worldbuilding.organization.placeholder.headquarters')
              "
            />
          </n-form-item>

          <n-form-item :label="t('worldbuilding.organization.memberCount')">
            <n-input-number
              v-model:value="organizationForm.member_count"
              :min="0"
              :placeholder="
                t('worldbuilding.organization.placeholder.memberCount')
              "
              style="width: 100%"
            />
          </n-form-item>

          <n-divider>{{ t("worldbuilding.customFields") }}</n-divider>

          <n-form-item :label="t('worldbuilding.addCustomField')">
            <n-dynamic-input
              v-model:value="organizationCustomFields"
              preset="pair"
              :key-placeholder="t('worldbuilding.fieldName')"
              :value-placeholder="t('worldbuilding.fieldValue')"
            />
          </n-form-item>
        </n-form>

        <template #footer>
          <n-space justify="end">
            <n-button @click="organizationDrawerVisible = false">{{
              t("worldbuilding.cancel")
            }}</n-button>
            <n-button type="primary" @click="handleSaveOrganization">{{
              t("worldbuilding.save")
            }}</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Character Detail Drawer (Read-only) -->
    <n-drawer
      v-model:show="characterDetailVisible"
      :width="480"
      placement="right"
      :trap-focus="true"
      :block-scroll="true"
    >
      <n-drawer-content
        :title="viewingCharacter?.name || t('worldbuilding.character.detail')"
        closable
      >
        <div v-if="viewingCharacter" class="space-y-6">
          <!-- Avatar & Basic Info -->
          <div
            class="flex flex-col items-center text-center pb-4 border-b border-gray-100 dark:border-gray-700"
          >
            <div
              class="w-20 h-20 rounded-full flex items-center justify-center text-2xl font-bold text-white mb-3"
              :class="
                viewingCharacter.gender === 'male'
                  ? 'bg-linear-to-br from-blue-500 to-blue-600'
                  : viewingCharacter.gender === 'female'
                  ? 'bg-linear-to-br from-pink-500 to-pink-600'
                  : 'bg-linear-to-br from-gray-500 to-gray-600'
              "
            >
              {{ viewingCharacter.name.charAt(0).toUpperCase() }}
            </div>
            <div class="flex items-center gap-2">
              <n-tag
                v-if="viewingCharacter.gender"
                :type="viewingCharacter.gender === 'male' ? 'info' : 'warning'"
                size="small"
              >
                {{ enumDictionary.getGenderName(viewingCharacter.gender) }}
              </n-tag>
              <n-tag v-if="viewingCharacter.age" type="default" size="small">
                {{ viewingCharacter.age }}{{ t("worldbuilding.ageUnit") }}
              </n-tag>
            </div>
          </div>

          <!-- Personality -->
          <div v-if="viewingCharacter.personality">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
              {{ t("worldbuilding.character.personalityTitle") }}
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed pl-3.5">
              {{ viewingCharacter.personality }}
            </p>
          </div>

          <!-- Appearance -->
          <div v-if="viewingCharacter.appearance">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-pink-500"></span>
              {{ t("worldbuilding.character.appearanceTitle") }}
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed pl-3.5">
              {{ viewingCharacter.appearance }}
            </p>
          </div>

          <!-- Background -->
          <div v-if="viewingCharacter.background">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-purple-500"></span>
              {{ t("worldbuilding.character.backgroundTitle") }}
            </h4>
            <p
              class="text-gray-700 dark:text-gray-200 leading-relaxed whitespace-pre-wrap pl-3.5"
            >
              {{ viewingCharacter.background }}
            </p>
          </div>

          <!-- Custom Fields -->
          <div v-if="viewingCharacterCustomFields.length > 0">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-amber-500"></span>
              {{ t("worldbuilding.customFields") }}
            </h4>
            <div class="space-y-2 pl-3.5">
              <div
                v-for="(field, index) in viewingCharacterCustomFields"
                :key="index"
                class="flex items-start gap-2"
              >
                <span class="text-gray-500 dark:text-gray-400 min-w-[60px]"
                  >{{ field.key }}:</span
                >
                <span class="text-gray-700 dark:text-gray-200">{{
                  field.value
                }}</span>
              </div>
            </div>
          </div>
        </div>

        <template #footer>
          <n-space justify="end">
            <n-button @click="characterDetailVisible = false">{{
              t("worldbuilding.close")
            }}</n-button>
            <n-button type="primary" @click="editCharacterFromDetail">{{
              t("worldbuilding.character.edit")
            }}</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Location Detail Drawer (Read-only) -->
    <n-drawer
      v-model:show="locationDetailVisible"
      :width="480"
      placement="right"
      :trap-focus="true"
      :block-scroll="true"
    >
      <n-drawer-content
        :title="viewingLocation?.name || t('worldbuilding.location.detail')"
        closable
      >
        <div v-if="viewingLocation" class="space-y-6">
          <!-- Location Icon & Basic Info -->
          <div
            class="flex flex-col items-center text-center pb-4 border-b border-gray-100 dark:border-gray-700"
          >
            <div
              class="w-20 h-20 rounded-full flex items-center justify-center text-2xl font-bold text-white mb-3 bg-linear-to-br from-green-500 to-green-600"
            >
              📍
            </div>
            <div class="flex items-center gap-2">
              <n-tag
                v-if="viewingLocation.location_type"
                type="success"
                size="small"
              >
                {{ getLocationTypeLabel(viewingLocation.location_type) }}
              </n-tag>
              <n-tag
                v-if="viewingLocation.population"
                type="default"
                size="small"
              >
                {{ viewingLocation.population
                }}{{ t("worldbuilding.peopleUnit") }}
              </n-tag>
            </div>
          </div>

          <!-- Climate -->
          <div v-if="viewingLocation.climate">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-cyan-500"></span>
              {{ t("worldbuilding.location.climateTitle") }}
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed pl-3.5">
              {{ viewingLocation.climate }}
            </p>
          </div>

          <!-- Description -->
          <div v-if="viewingLocation.description">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
              {{ t("worldbuilding.location.descriptionTitle") }}
            </h4>
            <p
              class="text-gray-700 dark:text-gray-200 leading-relaxed whitespace-pre-wrap pl-3.5"
            >
              {{ viewingLocation.description }}
            </p>
          </div>

          <!-- Notable Features -->
          <div v-if="viewingLocation.notable_features">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-amber-500"></span>
              {{ t("worldbuilding.location.notableFeaturesTitle") }}
            </h4>
            <p
              class="text-gray-700 dark:text-gray-200 leading-relaxed whitespace-pre-wrap pl-3.5"
            >
              {{ viewingLocation.notable_features }}
            </p>
          </div>

          <!-- Custom Fields -->
          <div v-if="viewingLocationCustomFields.length > 0">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-purple-500"></span>
              {{ t("worldbuilding.customFields") }}
            </h4>
            <div class="space-y-2 pl-3.5">
              <div
                v-for="(field, index) in viewingLocationCustomFields"
                :key="index"
                class="flex items-start gap-2"
              >
                <span class="text-gray-500 dark:text-gray-400 min-w-[60px]"
                  >{{ field.key }}:</span
                >
                <span class="text-gray-700 dark:text-gray-200">{{
                  field.value
                }}</span>
              </div>
            </div>
          </div>
        </div>

        <template #footer>
          <n-space justify="end">
            <n-button @click="locationDetailVisible = false">{{
              t("worldbuilding.close")
            }}</n-button>
            <n-button type="primary" @click="editLocationFromDetail">{{
              t("worldbuilding.location.edit")
            }}</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>

    <!-- Organization Detail Drawer (Read-only) -->
    <n-drawer
      v-model:show="organizationDetailVisible"
      :width="480"
      placement="right"
      :trap-focus="true"
      :block-scroll="true"
    >
      <n-drawer-content
        :title="
          viewingOrganization?.name || t('worldbuilding.organization.detail')
        "
        closable
      >
        <div v-if="viewingOrganization" class="space-y-6">
          <!-- Organization Icon & Basic Info -->
          <div
            class="flex flex-col items-center text-center pb-4 border-b border-gray-100 dark:border-gray-700"
          >
            <div
              class="w-20 h-20 rounded-full flex items-center justify-center text-2xl font-bold text-white mb-3 bg-linear-to-br from-orange-500 to-orange-600"
            >
              🏛️
            </div>
            <div class="flex items-center gap-2">
              <n-tag
                v-if="viewingOrganization.org_type"
                type="warning"
                size="small"
              >
                {{ getOrgTypeLabel(viewingOrganization.org_type) }}
              </n-tag>
              <n-tag
                v-if="viewingOrganization.member_count"
                type="default"
                size="small"
              >
                {{ viewingOrganization.member_count
                }}{{ t("worldbuilding.peopleUnit") }}
              </n-tag>
            </div>
          </div>

          <!-- Leader -->
          <div v-if="viewingOrganization.leader">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-red-500"></span>
              {{ t("worldbuilding.organization.leaderTitle") }}
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed pl-3.5">
              {{ viewingOrganization.leader }}
            </p>
          </div>

          <!-- Headquarters -->
          <div v-if="viewingOrganization.headquarters">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
              {{ t("worldbuilding.organization.headquartersTitle") }}
            </h4>
            <p class="text-gray-700 dark:text-gray-200 leading-relaxed pl-3.5">
              {{ viewingOrganization.headquarters }}
            </p>
          </div>

          <!-- Description -->
          <div v-if="viewingOrganization.description">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-purple-500"></span>
              {{ t("worldbuilding.organization.descriptionTitle") }}
            </h4>
            <p
              class="text-gray-700 dark:text-gray-200 leading-relaxed whitespace-pre-wrap pl-3.5"
            >
              {{ viewingOrganization.description }}
            </p>
          </div>

          <!-- Custom Fields -->
          <div v-if="viewingOrganizationCustomFields.length > 0">
            <h4
              class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 flex items-center gap-2"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-amber-500"></span>
              {{ t("worldbuilding.customFields") }}
            </h4>
            <div class="space-y-2 pl-3.5">
              <div
                v-for="(field, index) in viewingOrganizationCustomFields"
                :key="index"
                class="flex items-start gap-2"
              >
                <span class="text-gray-500 dark:text-gray-400 min-w-[60px]"
                  >{{ field.key }}:</span
                >
                <span class="text-gray-700 dark:text-gray-200">{{
                  field.value
                }}</span>
              </div>
            </div>
          </div>
        </div>

        <template #footer>
          <n-space justify="end">
            <n-button @click="organizationDetailVisible = false">{{
              t("worldbuilding.close")
            }}</n-button>
            <n-button type="primary" @click="editOrganizationFromDetail">{{
              t("worldbuilding.organization.edit")
            }}</n-button>
          </n-space>
        </template>
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import {
  NIcon,
  NButton,
  NTag,
  NTabs,
  NTabPane,
  NEmpty,
  NDivider,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NRadioGroup,
  NRadio,
  NSpace,
  NInputNumber,
  NDynamicInput,
  NDrawer,
  NDrawerContent,
} from "naive-ui";
import { Plus, Trash, BookOpen } from "lucide-vue-next";
import {
  useWorldbuildingStore,
  type Character,
  type Location,
  type Organization,
  type CustomField,
} from "@/stores/worldbuilding";
import { useProjectStore } from "@/stores/project";
import { useEnumDictionary } from "@/stores/enumDictionary";
import { useLocale } from "@/i18n/composables/useLocale";

const store = useWorldbuildingStore();
const projectStore = useProjectStore();
const enumDictionary = useEnumDictionary();
const { t } = useLocale();

// 初始化时加载字典数据
enumDictionary.loadDictionary();

// Get current project ID
const getProjectId = () => projectStore.currentProject?.id ?? null;

const activeTab = ref("characters");

// Drawer states
const characterDrawerVisible = ref(false);
const locationDrawerVisible = ref(false);
const organizationDrawerVisible = ref(false);

// Editing states
const editingCharacter = ref<Character | null>(null);
const editingLocation = ref<Location | null>(null);
const editingOrganization = ref<Organization | null>(null);

// Character detail view states (read-only)
const characterDetailVisible = ref(false);
const viewingCharacter = ref<Character | null>(null);
const viewingCharacterCustomFields = ref<CustomField[]>([]);

// Location detail view states (read-only)
const locationDetailVisible = ref(false);
const viewingLocation = ref<Location | null>(null);
const viewingLocationCustomFields = ref<CustomField[]>([]);

// Organization detail view states (read-only)
const organizationDetailVisible = ref(false);
const viewingOrganization = ref<Organization | null>(null);
const viewingOrganizationCustomFields = ref<CustomField[]>([]);

// Form states
const characterForm = reactive({
  name: "",
  gender: "",
  age: null as number | null,
  appearance: "",
  personality: "",
  background: "",
});

const showGenderError = ref(false);

const characterCustomFields = ref<CustomField[]>([]);

const locationForm = reactive({
  name: "",
  location_type: "",
  description: "",
  climate: "",
  population: null as number | null,
  notable_features: "",
});

const locationCustomFields = ref<CustomField[]>([]);

const organizationForm = reactive({
  name: "",
  org_type: "",
  description: "",
  leader: "",
  headquarters: "",
  member_count: null as number | null,
});

const organizationCustomFields = ref<CustomField[]>([]);

// Helper functions
function getLocationTypeLabel(type: string): string {
  return enumDictionary.getLocationTypeName(type);
}

function getOrgTypeLabel(type: string): string {
  return enumDictionary.getOrganizationTypeName(type);
}

function parseCustomFields(jsonStr: string): CustomField[] {
  try {
    const parsed = JSON.parse(jsonStr);
    return Object.entries(parsed).map(([key, value]) => ({
      key,
      value: String(value),
    }));
  } catch {
    return [];
  }
}

function stringifyCustomFields(fields: CustomField[]): string {
  const obj: Record<string, string> = {};
  fields.forEach((f) => {
    if (f.key.trim()) {
      obj[f.key.trim()] = f.value;
    }
  });
  return JSON.stringify(obj);
}

function resetCharacterForm() {
  editingCharacter.value = null;
  characterForm.name = "";
  characterForm.gender = "";
  characterForm.age = null;
  characterForm.appearance = "";
  characterForm.personality = "";
  characterForm.background = "";
  characterCustomFields.value = [];
  showGenderError.value = false;
}

function resetLocationForm() {
  editingLocation.value = null;
  locationForm.name = "";
  locationForm.location_type = "";
  locationForm.description = "";
  locationForm.climate = "";
  locationForm.population = null;
  locationForm.notable_features = "";
  locationCustomFields.value = [];
}

function resetOrganizationForm() {
  editingOrganization.value = null;
  organizationForm.name = "";
  organizationForm.org_type = "";
  organizationForm.description = "";
  organizationForm.leader = "";
  organizationForm.headquarters = "";
  organizationForm.member_count = null;
  organizationCustomFields.value = [];
}

// Open drawer functions
function openCharacterDrawer(character?: Character) {
  if (character) {
    editingCharacter.value = character;
    characterForm.name = character.name;
    characterForm.gender = character.gender;
    characterForm.age = character.age;
    characterForm.appearance = character.appearance;
    characterForm.personality = character.personality;
    characterForm.background = character.background;
    characterCustomFields.value = parseCustomFields(character.custom_fields);
  } else {
    resetCharacterForm();
  }
  characterDrawerVisible.value = true;
}

// View character detail (read-only)
function viewCharacterDetail(character: Character) {
  activeTab.value = "characters";
  viewingCharacter.value = character;
  viewingCharacterCustomFields.value = parseCustomFields(
    character.custom_fields
  );
  characterDetailVisible.value = true;
}

// View location detail (read-only)
function viewLocationDetail(location: Location) {
  activeTab.value = "locations";
  viewingLocation.value = location;
  viewingLocationCustomFields.value = parseCustomFields(location.custom_fields);
  locationDetailVisible.value = true;
}

// View organization detail (read-only)
function viewOrganizationDetail(organization: Organization) {
  activeTab.value = "organizations";
  viewingOrganization.value = organization;
  viewingOrganizationCustomFields.value = parseCustomFields(
    organization.custom_fields
  );
  organizationDetailVisible.value = true;
}

// Edit character from detail view
function editCharacterFromDetail() {
  if (viewingCharacter.value) {
    characterDetailVisible.value = false;
    openCharacterDrawer(viewingCharacter.value);
  }
}

// Edit location from detail view
function editLocationFromDetail() {
  if (viewingLocation.value) {
    locationDetailVisible.value = false;
    openLocationDrawer(viewingLocation.value);
  }
}

// Edit organization from detail view
function editOrganizationFromDetail() {
  if (viewingOrganization.value) {
    organizationDetailVisible.value = false;
    openOrganizationDrawer(viewingOrganization.value);
  }
}

function openLocationDrawer(location?: Location) {
  if (location) {
    editingLocation.value = location;
    locationForm.name = location.name;
    locationForm.location_type = location.location_type;
    locationForm.description = location.description;
    locationForm.climate = location.climate;
    locationForm.population = location.population;
    locationForm.notable_features = location.notable_features;
    locationCustomFields.value = parseCustomFields(location.custom_fields);
  } else {
    resetLocationForm();
  }
  locationDrawerVisible.value = true;
}

function openOrganizationDrawer(org?: Organization) {
  if (org) {
    editingOrganization.value = org;
    organizationForm.name = org.name;
    organizationForm.org_type = org.org_type;
    organizationForm.description = org.description;
    organizationForm.leader = org.leader;
    organizationForm.headquarters = org.headquarters;
    organizationForm.member_count = org.member_count;
    organizationCustomFields.value = parseCustomFields(org.custom_fields);
  } else {
    resetOrganizationForm();
  }
  organizationDrawerVisible.value = true;
}

// Save functions
async function handleSaveCharacter() {
  // 重置错误状态
  showGenderError.value = false;

  // 验证姓名
  if (!characterForm.name.trim()) {
    return;
  }

  // 验证性别
  if (!characterForm.gender) {
    showGenderError.value = true;
    return;
  }

  const projectId = getProjectId();
  if (!projectId) {
    alert(t("worldbuilding.alert.openProject"));
    return;
  }

  const customFieldsStr = stringifyCustomFields(characterCustomFields.value);

  if (editingCharacter.value) {
    await store.updateCharacter(editingCharacter.value.id, {
      name: characterForm.name,
      gender: characterForm.gender,
      age: characterForm.age,
      appearance: characterForm.appearance,
      personality: characterForm.personality,
      background: characterForm.background,
      custom_fields: customFieldsStr,
    });
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
    });
  }

  characterDrawerVisible.value = false;
}

async function handleSaveLocation() {
  if (!locationForm.name.trim()) {
    return;
  }

  const projectId = getProjectId();
  if (!projectId) {
    alert(t("worldbuilding.alert.openProject"));
    return;
  }

  const customFieldsStr = stringifyCustomFields(locationCustomFields.value);

  if (editingLocation.value) {
    await store.updateLocation(editingLocation.value.id, {
      name: locationForm.name,
      location_type: locationForm.location_type,
      description: locationForm.description,
      climate: locationForm.climate,
      population: locationForm.population,
      notable_features: locationForm.notable_features,
      custom_fields: customFieldsStr,
    });
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
    });
  }

  locationDrawerVisible.value = false;
}

async function handleSaveOrganization() {
  if (!organizationForm.name.trim()) {
    return;
  }

  const projectId = getProjectId();
  if (!projectId) {
    alert(t("worldbuilding.alert.openProject"));
    return;
  }

  const customFieldsStr = stringifyCustomFields(organizationCustomFields.value);

  if (editingOrganization.value) {
    await store.updateOrganization(editingOrganization.value.id, {
      name: organizationForm.name,
      org_type: organizationForm.org_type,
      description: organizationForm.description,
      leader: organizationForm.leader,
      headquarters: organizationForm.headquarters,
      member_count: organizationForm.member_count,
      custom_fields: customFieldsStr,
    });
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
    });
  }

  organizationDrawerVisible.value = false;
}

// Delete functions
async function handleDeleteCharacter(id: number) {
  await store.deleteCharacter(id);
}

async function handleDeleteLocation(id: number) {
  await store.deleteLocation(id);
}

async function handleDeleteOrganization(id: number) {
  await store.deleteOrganization(id);
}

// Load data on mount
onMounted(async () => {
  const projectId = getProjectId();
  if (projectId) {
    await store.loadAll(projectId);
  }
});

// Expose methods to parent component
defineExpose({
  viewCharacterDetail,
  viewLocationDetail,
  viewOrganizationDetail,
  openCharacterDrawer,
  openLocationDrawer,
  openOrganizationDrawer,
});
</script>

<style scoped>
:deep(.n-tab-pane) {
  padding: 16px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

:deep(.n-tab-pane > div[class*="space-y-"]) > * + * {
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

  :deep(.n-tab-pane > div[class*="space-y-"]) > * + * {
    margin-top: 8px;
  }

  :deep(.worldbuilding-tabs .n-tabs-tab) {
    padding-left: 8px !important;
    padding-right: 8px !important;
    font-size: 11px !important;
  }
}

@media screen and (min-width: 641px) and (max-width: 1024px) {
  :deep(.n-tab-pane > div[class*="space-y-"]) > * + * {
    margin-top: 10px;
  }
}

/* 性别单选按钮组样式修复 */
:deep(.gender-radio-group) {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

:deep(.gender-radio-group .n-radio-button) {
  height: 34px;
  padding: 0 16px;
  background-color: #fff !important;
  border: 2px solid #ddd !important;
  border-radius: 6px;
  font-size: 14px;
  line-height: 30px;
  transition: all 0.2s ease;
  box-shadow: none !important;
}

:deep(.gender-radio-group .n-radio-button .n-radio-button__content) {
  color: #333 !important;
}

:deep(
    .gender-radio-group
      .n-radio-button
      .n-radio-button__content
      .n-radio-button__label
  ) {
  color: #333 !important;
}

:deep(.gender-radio-group .n-radio-button:hover) {
  border-color: #63e6be !important;
}

:deep(
    .gender-radio-group
      .n-radio-button:hover
      .n-radio-button__content
      .n-radio-button__label
  ) {
  color: #18a058 !important;
}

:deep(.gender-radio-group .n-radio-button--checked) {
  background-color: #63e6be !important;
  border-color: #63e6be !important;
}

:deep(
    .gender-radio-group
      .n-radio-button--checked
      .n-radio-button__content
      .n-radio-button__label
  ) {
  color: #fff !important;
  font-weight: 500;
}

/* 暗色模式支持 */
:deep(.n-theme-dark) .gender-radio-group .n-radio-button {
  background-color: #333 !important;
  border: 2px solid #555 !important;
}

:deep(.n-theme-dark)
  .gender-radio-group
  .n-radio-button
  .n-radio-button__content
  .n-radio-button__label {
  color: #fff !important;
}

:deep(.n-theme-dark) .gender-radio-group .n-radio-button--checked {
  background-color: #63e6be !important;
  border-color: #63e6be !important;
}

:deep(.n-theme-dark)
  .gender-radio-group
  .n-radio-button--checked
  .n-radio-button__content
  .n-radio-button__label {
  color: #000 !important;
}
</style>
