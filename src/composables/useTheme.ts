import { computed } from "vue";
import { useDark, useToggle } from "@vueuse/core";
import { darkTheme } from "naive-ui";
import type { GlobalTheme } from "naive-ui";

const isDark = useDark({
  selector: "html",
  attribute: "class",
  valueDark: "dark",
  valueLight: "",
});

const toggleDark = useToggle(isDark);

const theme = computed<GlobalTheme | undefined>(() =>
  isDark.value ? darkTheme : undefined
);

export function useTheme() {
  return {
    isDark,
    toggleDark,
    theme,
  };
}
