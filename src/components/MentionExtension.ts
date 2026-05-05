import Mention from '@tiptap/extension-mention'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import { useWorldbuildingStore } from '../stores/worldbuilding'
import MentionNodeComponent from './MentionNode.vue'

export interface MentionItem {
  id: string
  label: string
  type: 'character' | 'location' | 'organization'
}

function escapeHtml(str: string) {
  const el = document.createElement('span')
  el.textContent = str
  return el.innerHTML
}

export function createMentionExtension() {
  return Mention
    .extend({
      addNodeView() {
        return VueNodeViewRenderer(MentionNodeComponent)
      },
    })
    .configure({
      HTMLAttributes: { class: 'mention' },
      suggestion: {
        items: ({ query }) => {
          const store = useWorldbuildingStore()
          const results: MentionItem[] = []
          const q = query.toLowerCase()
          store.characters.forEach(c => {
            if (c.name.toLowerCase().includes(q)) {
              results.push({ id: `character-${c.id}`, label: c.name, type: 'character' })
            }
          })
          store.locations.forEach(l => {
            if (l.name.toLowerCase().includes(q)) {
              results.push({ id: `location-${l.id}`, label: l.name, type: 'location' })
            }
          })
          store.organizations.forEach(o => {
            if (o.name.toLowerCase().includes(q)) {
              results.push({ id: `organization-${o.id}`, label: o.name, type: 'organization' })
            }
          })
          return results.slice(0, 10)
        },
        render: () => {
          let dom: HTMLDivElement | null = null

          function updatePos() {
            if (!dom) return
            try {
              const sel = window.getSelection()
              if (sel && sel.rangeCount) {
                const rect = sel.getRangeAt(0).getBoundingClientRect()
                if (rect) {
                  dom.style.top = `${rect.bottom + 4}px`
                  dom.style.left = `${Math.max(0, rect.left)}px`
                }
              }
            } catch { /* ignore */ }
          }

          function renderItems(items: MentionItem[]) {
            if (!dom) return
            dom.innerHTML = ''
            const dark = document.documentElement.classList.contains('dark')
            const labelColor = dark ? '#f3f4f6' : '#1f2937'
            const bgColor = dark ? '#1f2937' : '#fff'
            const borderColor = dark ? '#374151' : '#e5e7eb'
            const selBg = dark ? 'rgba(59,130,246,0.3)' : '#e0e7ff'

            Object.assign(dom.style, {
              background: bgColor,
              border: `1px solid ${borderColor}`,
              boxShadow: dark
                ? '0 10px 40px rgba(0,0,0,0.5)'
                : '0 10px 40px rgba(0,0,0,0.15)',
            })

            if (items.length === 0) {
              const empty = document.createElement('div')
              empty.style.cssText = `padding:12px 16px;font-size:14px;text-align:center;color:${dark ? '#6b7280' : '#9ca3af'}`
              empty.textContent = '无匹配结果'
              dom.appendChild(empty)
              return
            }

            items.forEach((item, idx) => {
              const row = document.createElement('div')
              row.style.cssText = `display:flex;align-items:center;gap:8px;padding:8px 12px;border-radius:6px;cursor:pointer;${idx === 0 ? `background:${selBg}` : ''}`
              row.innerHTML = `
                <span>${item.type === 'character' ? '👤' : item.type === 'location' ? '📍' : '🏛️'}</span>
                <span style="flex:1;font-weight:500;color:${labelColor}">${escapeHtml(item.label)}</span>
                <span style="font-size:11px;padding:0 6px;border-radius:3px;background:${item.type === 'character' ? '#3b82f6' : item.type === 'location' ? '#22c55e' : '#f59e0b'};color:#fff">${item.type === 'character' ? '人物' : item.type === 'location' ? '地点' : '组织'}</span>
              `
              row.addEventListener('mousedown', (e) => { e.preventDefault(); e.stopPropagation() })
              row.addEventListener('click', () => {
                props.editor
                  .chain()
                  .focus()
                  .insertContentAt(props.range, [
                    { type: 'mention', attrs: { id: item.id, label: item.label } },
                    { type: 'text', text: ' ' },
                  ])
                  .run()
                dom?.remove()
                dom = null
              })
              row.addEventListener('mouseenter', () => {
                if (dom) {
                  dom.querySelectorAll('div').forEach(d => d.style.background = '')
                }
                row.style.background = selBg
              })
              if (dom) {
                dom.appendChild(row)
              }
            })
          }

          let props: { editor: any; range: { from: number; to: number } }

          return {
            onStart: (p: any) => {
              props = p
              dom = document.createElement('div')
              Object.assign(dom.style, {
                position: 'fixed', zIndex: '10000',
                borderRadius: '8px', maxHeight: '300px', overflowY: 'auto',
                minWidth: '200px', padding: '4px',
              })
              dom.addEventListener('mousedown', (e) => { e.preventDefault(); e.stopPropagation() })
              document.body.appendChild(dom)
              renderItems(p.items)
              updatePos()
            },
            onUpdate: (p: any) => {
              props = p
              renderItems(p.items)
              updatePos()
            },
            onKeyDown: () => false,
            onExit: () => { dom?.remove(); dom = null },
          }
        },
      },
    })
}

export function parseMentionId(id: string) {
  const m = id.match(/^(character|location|organization)-(\d+)$/)
  return m
    ? { type: m[1] as 'character' | 'location' | 'organization', numericId: parseInt(m[2], 10) }
    : { type: null, numericId: null }
}

export function getMentionData(id: string) {
  const { type, numericId } = parseMentionId(id)
  if (!type || !numericId) return null
  const store = useWorldbuildingStore()
  switch (type) {
    case 'character': return store.getCharacterById(numericId)
    case 'location': return store.getLocationById(numericId)
    case 'organization': return store.getOrganizationById(numericId)
    default: return null
  }
}
