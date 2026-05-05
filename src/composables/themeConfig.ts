import type { GlobalThemeOverrides } from "naive-ui";

// 亮色主题覆盖配置
export const lightThemeConfig: GlobalThemeOverrides = {
  common: {
    primaryColor: "#18a058",
    primaryColorHover: "#36ad6a",
    primaryColorPressed: "#0c7a43",
    bodyColor: "#ffffff",
    cardColor: "#ffffff",
    modalColor: "#ffffff",
    popoverColor: "#ffffff",
    tableColor: "#ffffff",
    inputColor: "#ffffff",
    actionColor: "#ffffff",
  },
};

// 深色主题覆盖配置
export const darkThemeConfig: GlobalThemeOverrides = {
  common: {
    primaryColor: "#63e6be",
    primaryColorHover: "#69f0ae",
    primaryColorPressed: "#38d9a9",
    bodyColor: "#1a1a1a",
    cardColor: "#242424",
    modalColor: "#242424",
    popoverColor: "#242424",
    tableColor: "#242424",
    inputColor: "#333333",
    actionColor: "#333333",
  },
};
