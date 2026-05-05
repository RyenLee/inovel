import { defineStore } from "pinia";
import { ref, computed } from "vue";

export type EditorMode = "normal" | "typewriter" | "focus";

export const useEditorStore = defineStore("editor", () => {
  const mode = ref<EditorMode>("normal");

  const isNormal = computed(() => mode.value === "normal");
  const isTypewriter = computed(() => mode.value === "typewriter");
  const isFocus = computed(() => mode.value === "focus");

  function setMode(newMode: EditorMode) {
    mode.value = newMode;
  }

  function toggleTypewriter() {
    mode.value = mode.value === "typewriter" ? "normal" : "typewriter";
  }

  function toggleFocus() {
    mode.value = mode.value === "focus" ? "normal" : "focus";
  }

  function exitSpecialMode() {
    if (mode.value !== "normal") {
      mode.value = "normal";
    }
  }

  return {
    mode,
    isNormal,
    isTypewriter,
    isFocus,
    setMode,
    toggleTypewriter,
    toggleFocus,
    exitSpecialMode,
  };
});
