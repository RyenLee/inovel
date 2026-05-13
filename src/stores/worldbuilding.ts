import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface Character {
  id: number
  project_id: number
  name: string
  gender: string
  age: number | null
  appearance: string
  personality: string
  background: string
  custom_fields: string
  created_at: string
  updated_at: string
}

export interface CreateCharacterParams {
  project_id: number
  name: string
  gender: string
  age: number | null
  appearance: string
  personality: string
  background: string
  custom_fields: string
}

export interface UpdateCharacterParams {
  name: string
  gender: string
  age: number | null
  appearance: string
  personality: string
  background: string
  custom_fields: string
}

export interface Location {
  id: number
  project_id: number
  name: string
  location_type: string
  description: string
  climate: string
  population: number | null
  notable_features: string
  custom_fields: string
  created_at: string
  updated_at: string
}

export interface CreateLocationParams {
  project_id: number
  name: string
  location_type: string
  description: string
  climate: string
  population: number | null
  notable_features: string
  custom_fields: string
}

export interface UpdateLocationParams {
  name: string
  location_type: string
  description: string
  climate: string
  population: number | null
  notable_features: string
  custom_fields: string
}

export interface Organization {
  id: number
  project_id: number
  name: string
  org_type: string
  description: string
  leader: string
  headquarters: string
  member_count: number | null
  custom_fields: string
  created_at: string
  updated_at: string
}

export interface CreateOrganizationParams {
  project_id: number
  name: string
  org_type: string
  description: string
  leader: string
  headquarters: string
  member_count: number | null
  custom_fields: string
}

export interface UpdateOrganizationParams {
  name: string
  org_type: string
  description: string
  leader: string
  headquarters: string
  member_count: number | null
  custom_fields: string
}

export interface CustomField {
  key: string
  value: string
}

export const useWorldbuildingStore = defineStore('worldbuilding', {
  state: () => ({
    characters: [] as Character[],
    locations: [] as Location[],
    organizations: [] as Organization[],
    loading: false,
    currentProjectId: null as number | null,
  }),

  getters: {
    getCharacterById: (state) => (id: number) => {
      return state.characters.find(c => c.id === id)
    },
    getLocationById: (state) => (id: number) => {
      return state.locations.find(l => l.id === id)
    },
    getOrganizationById: (state) => (id: number) => {
      return state.organizations.find(o => o.id === id)
    },
    parseCustomFields: () => (jsonStr: string): CustomField[] => {
      try {
        const parsed = JSON.parse(jsonStr)
        return Object.entries(parsed).map(([key, value]) => ({ key, value: String(value) }))
      } catch {
        return []
      }
    },
    stringifyCustomFields: () => (fields: CustomField[]): string => {
      const obj: Record<string, string> = {}
      fields.forEach(f => {
        if (f.key.trim()) {
          obj[f.key.trim()] = f.value
        }
      })
      return JSON.stringify(obj)
    },
  },

  actions: {
    async loadAll(projectId: number) {
      this.loading = true
      this.currentProjectId = projectId
      try {
        await Promise.all([
          this.loadCharacters(projectId),
          this.loadLocations(projectId),
          this.loadOrganizations(projectId),
        ])
      } finally {
        this.loading = false
      }
    },

    // Character actions
    async loadCharacters(projectId: number) {
      try {
        this.characters = await invoke<Character[]>('list_characters', { project_id: projectId })
      } catch (error) {
        console.error('加载角色列表失败:', error)
        this.characters = []
      }
    },

    async createCharacter(params: CreateCharacterParams): Promise<Character | null> {
      try {
        const character = await invoke<Character>('create_character', { params })
        this.characters.unshift(character)
        return character
      } catch (error) {
        console.error('创建角色失败:', error)
        return null
      }
    },

    async updateCharacter(characterId: number, params: UpdateCharacterParams): Promise<Character | null> {
      try {
        const character = await invoke<Character>('update_character', { character_id: characterId, params })
        const index = this.characters.findIndex(c => c.id === characterId)
        if (index !== -1) {
          this.characters[index] = character
        }
        return character
      } catch (error) {
        console.error('更新角色失败:', error)
        return null
      }
    },

    async deleteCharacter(characterId: number): Promise<boolean> {
      try {
        await invoke('delete_character', { character_id: characterId })
        this.characters = this.characters.filter(c => c.id !== characterId)
        return true
      } catch (error) {
        console.error('删除角色失败:', error)
        return false
      }
    },

    // Location actions
    async loadLocations(projectId: number) {
      try {
        this.locations = await invoke<Location[]>('list_locations', { project_id: projectId })
      } catch (error) {
        console.error('加载地点列表失败:', error)
        this.locations = []
      }
    },

    async createLocation(params: CreateLocationParams): Promise<Location | null> {
      try {
        const location = await invoke<Location>('create_location', { params })
        this.locations.unshift(location)
        return location
      } catch (error) {
        console.error('创建地点失败:', error)
        return null
      }
    },

    async updateLocation(locationId: number, params: UpdateLocationParams): Promise<Location | null> {
      try {
        const location = await invoke<Location>('update_location', { location_id: locationId, params })
        const index = this.locations.findIndex(l => l.id === locationId)
        if (index !== -1) {
          this.locations[index] = location
        }
        return location
      } catch (error) {
        console.error('更新地点失败:', error)
        return null
      }
    },

    async deleteLocation(locationId: number): Promise<boolean> {
      try {
        await invoke('delete_location', { location_id: locationId })
        this.locations = this.locations.filter(l => l.id !== locationId)
        return true
      } catch (error) {
        console.error('删除地点失败:', error)
        return false
      }
    },

    // Organization actions
    async loadOrganizations(projectId: number) {
      try {
        this.organizations = await invoke<Organization[]>('list_organizations', { project_id: projectId })
      } catch (error) {
        console.error('加载组织列表失败:', error)
        this.organizations = []
      }
    },

    async createOrganization(params: CreateOrganizationParams): Promise<Organization | null> {
      try {
        const organization = await invoke<Organization>('create_organization', { params })
        this.organizations.unshift(organization)
        return organization
      } catch (error) {
        console.error('创建组织失败:', error)
        return null
      }
    },

    async updateOrganization(organizationId: number, params: UpdateOrganizationParams): Promise<Organization | null> {
      try {
        const organization = await invoke<Organization>('update_organization', { organization_id: organizationId, params })
        const index = this.organizations.findIndex(o => o.id === organizationId)
        if (index !== -1) {
          this.organizations[index] = organization
        }
        return organization
      } catch (error) {
        console.error('更新组织失败:', error)
        return null
      }
    },

    async deleteOrganization(organizationId: number): Promise<boolean> {
      try {
        await invoke('delete_organization', { organization_id: organizationId })
        this.organizations = this.organizations.filter(o => o.id !== organizationId)
        return true
      } catch (error) {
        console.error('删除组织失败:', error)
        return false
      }
    },
  },
})
