import { onMounted, onUnmounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useShortcutStore } from "../stores/shortcuts";
import { useTheme } from "./useTheme";

// Action callbacks type
type ActionCallback = () => void;

export function useGlobalShortcuts() {
  const router = useRouter();
  const route = useRoute();
  const shortcutStore = useShortcutStore();
  const { toggleDark } = useTheme();

  // Action handlers
  const handlers: Record<string, ActionCallback> = {
    save: () => {
      // Dispatch save event to EditorPage
      window.dispatchEvent(new CustomEvent("shortcut:save"));
    },
    new_chapter: () => {
      window.dispatchEvent(new CustomEvent("shortcut:new-chapter"));
    },
    export: () => {
      window.dispatchEvent(new CustomEvent("shortcut:export"));
    },
    backup: () => {
      window.dispatchEvent(new CustomEvent("shortcut:backup"));
    },
    typewriter: () => {
      window.dispatchEvent(new CustomEvent("shortcut:typewriter"));
    },
    focus: () => {
      window.dispatchEvent(new CustomEvent("shortcut:focus"));
    },
    zen: () => {
      window.dispatchEvent(new CustomEvent("shortcut:zen"));
    },
    fullscreen: () => {
      window.dispatchEvent(new CustomEvent("shortcut:fullscreen"));
    },
    prev_chapter: () => {
      window.dispatchEvent(new CustomEvent("shortcut:prev-chapter"));
    },
    next_chapter: () => {
      window.dispatchEvent(new CustomEvent("shortcut:next-chapter"));
    },
    toggle_sidebar: () => {
      window.dispatchEvent(new CustomEvent("shortcut:toggle-sidebar"));
    },
    toggle_worldbuilding: () => {
      window.dispatchEvent(new CustomEvent("shortcut:toggle-worldbuilding"));
    },
    toggle_theme: () => {
      toggleDark();
    },
    show_stats: () => {
      if (route.params.projectId) {
        router.push(`/editor/${route.params.projectId}/project-stats`);
      }
    },
    show_settings: () => {
      if (route.params.projectId) {
        router.push(`/editor/${route.params.projectId}/project-settings`);
      }
    },
    snapshot: () => {
      window.dispatchEvent(new CustomEvent("shortcut:snapshot"));
    },
    name_generator: () => {
      window.dispatchEvent(new CustomEvent("shortcut:name-generator"));
    },
    sensitive_words: () => {
      window.dispatchEvent(new CustomEvent("shortcut:sensitive-words"));
    },
  };

  // Global keydown handler
  const handleKeyDown = (event: globalThis.KeyboardEvent) => {
    // Skip if user is typing in an input field
    const target = event.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
      return;
    }

    // Find matching shortcut
    for (const shortcut of shortcutStore.shortcuts) {
      if (shortcutStore.matchShortcut(shortcut, event)) {
        event.preventDefault();
        const handler = handlers[shortcut.id];
        if (handler) {
          handler();
        }
        return;
      }
    }
  };

  onMounted(() => {
    window.addEventListener("keydown", handleKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeyDown);
  });

  return {
    handlers,
  };
}
