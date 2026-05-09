import { useEntryConfigStore } from "../stores/entryConfig";
import { onMounted, onUnmounted } from "vue";

export function useEntryService() {
  const entryConfigStore = useEntryConfigStore();

  const openConfigPage = () => {
    window.dispatchEvent(new CustomEvent("openConfigPage"));
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (!entryConfigStore.isEntryEnabled) return;
    if (!entryConfigStore.hasLocation("keyboard")) return;

    const target = event.target as HTMLElement;
    if (
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.isContentEditable
    ) {
      return;
    }

    const { shortcut_modifiers, shortcut_key } = entryConfigStore.config;
    if (!shortcut_key) return;

    const modifiersMatch =
      shortcut_modifiers.includes("Ctrl") === (event.ctrlKey || event.metaKey) &&
      shortcut_modifiers.includes("Shift") === event.shiftKey &&
      shortcut_modifiers.includes("Alt") === event.altKey;

    if (modifiersMatch && event.key.toLowerCase() === shortcut_key.toLowerCase()) {
      event.preventDefault();
      openConfigPage();
    }
  };

  const setupKeyboardEntry = () => {
    window.addEventListener("keydown", handleKeyDown);
  };

  const cleanupKeyboardEntry = () => {
    window.removeEventListener("keydown", handleKeyDown);
  };

  const handleConfigUpdated = () => {
    cleanupKeyboardEntry();
    if (entryConfigStore.hasLocation("keyboard")) {
      setupKeyboardEntry();
    }
  };

  onMounted(() => {
    if (entryConfigStore.hasLocation("keyboard")) {
      setupKeyboardEntry();
    }
    window.addEventListener("entryConfigUpdated", handleConfigUpdated);
  });

  onUnmounted(() => {
    cleanupKeyboardEntry();
    window.removeEventListener("entryConfigUpdated", handleConfigUpdated);
  });

  return {
    openConfigPage,
    isEntryEnabled: () => entryConfigStore.isEntryEnabled,
    hasLocation: (location: string) => entryConfigStore.hasLocation(location as any),
    getConfig: () => entryConfigStore.config,
  };
}

export function createEntryButton() {
  const entryConfigStore = useEntryConfigStore();
  
  return {
    displayName: entryConfigStore.config.display_name,
    icon: entryConfigStore.config.icon,
    tooltip: entryConfigStore.config.tooltip,
    onClick: () => window.dispatchEvent(new CustomEvent("openConfigPage")),
    isVisible: entryConfigStore.isEntryEnabled,
  };
}