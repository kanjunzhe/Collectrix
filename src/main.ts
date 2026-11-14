import { createApp } from "vue";
import App from "./App.vue";
import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";
import "./styles/tailwind.css";
import { createVuetify } from "vuetify";
import {
  VAlert,
  VApp,
  VAppBar,
  VAvatar,
  VBtn,
  VBtnGroup,
  VBtnToggle,
  VCard,
  VCardActions,
  VCardSubtitle,
  VCardText,
  VCardTitle,
  VCheckbox,
  VCheckboxBtn,
  VChip,
  VCol,
  VCombobox,
  VContainer,
  VDialog,
  VDivider,
  VExpandTransition,
  VExpansionPanel,
  VExpansionPanels,
  VExpansionPanelText,
  VExpansionPanelTitle,
  VIcon,
  VImg,
  VList,
  VListGroup,
  VListItem,
  VListItemSubtitle,
  VListItemTitle,
  VMain,
  VMenu,
  VNavigationDrawer,
  VProgressCircular,
  VProgressLinear,
  VRow,
  VSelect,
  VSheet,
  VSnackbar,
  VSpacer,
  VSwitch,
  VTextarea,
  VTextField,
} from "vuetify/components";
import * as directives from "vuetify/directives";

const components = {
  VAlert,
  VApp,
  VAppBar,
  VAvatar,
  VBtn,
  VBtnGroup,
  VBtnToggle,
  VCard,
  VCardActions,
  VCardSubtitle,
  VCardText,
  VCardTitle,
  VCheckbox,
  VCheckboxBtn,
  VChip,
  VCol,
  VCombobox,
  VContainer,
  VDialog,
  VDivider,
  VExpandTransition,
  VExpansionPanel,
  VExpansionPanels,
  VExpansionPanelText,
  VExpansionPanelTitle,
  VIcon,
  VImg,
  VList,
  VListGroup,
  VListItem,
  VListItemSubtitle,
  VListItemTitle,
  VMain,
  VMenu,
  VNavigationDrawer,
  VProgressCircular,
  VProgressLinear,
  VRow,
  VSelect,
  VSheet,
  VSnackbar,
  VSpacer,
  VSwitch,
  VTextarea,
  VTextField,
};
import { initialTheme } from "./themes";

performance.mark("boot:entry");

const baseLightColors = {
  background: "#F5F7FA",
  surface: "#FFFFFF",
  "surface-variant": "#E8EDF5",
  "on-surface": "#1B1B1F",
  "on-background": "#1B1B1F",
  primary: "#1976D2",
  "on-primary": "#FFFFFF",
  secondary: "#00897B",
  "on-secondary": "#FFFFFF",
  success: "#2E7D32",
  warning: "#ED6C02",
  error: "#C62828",
  info: "#0277BD",
};

const darkColors = {
  background: "#121418",
  surface: "#1C2026",
  "surface-variant": "#2A3038",
  "on-surface": "#F2F5F8",
  "on-background": "#F2F5F8",
  primary: "#90CAF9",
  "on-primary": "#0D1B2A",
  secondary: "#80CBC4",
  "on-secondary": "#0D1B2A",
  success: "#81C784",
  warning: "#FFB74D",
  error: "#EF9A9A",
  info: "#4FC3F7",
};

const vuetify = createVuetify({
  components,
  directives,
  theme: {
    // 先用上次的主题渲染，真正的设置项由 App.vue 在拿到 get_settings 后校正。
    defaultTheme: initialTheme(),
    themes: {
      light: { dark: false, colors: { ...baseLightColors } },
      "light-blue": {
        dark: false,
        colors: {
          ...baseLightColors,
          background: "#F2F7FF",
          "surface-variant": "#E3EEFF",
          primary: "#1E88E5",
          secondary: "#64B5F6",
        },
      },
      "light-green": {
        dark: false,
        colors: {
          ...baseLightColors,
          background: "#F2FBF4",
          "surface-variant": "#E1F3E4",
          primary: "#43A047",
          secondary: "#81C784",
        },
      },
      "light-red": {
        dark: false,
        colors: {
          ...baseLightColors,
          background: "#FFF5F5",
          "surface-variant": "#FBE3E3",
          primary: "#E53935",
          secondary: "#EF9A9A",
        },
      },
      "light-purple": {
        dark: false,
        colors: {
          ...baseLightColors,
          background: "#FAF5FF",
          "surface-variant": "#F0E3FA",
          primary: "#8E24AA",
          secondary: "#BA68C8",
        },
      },
      "light-amber": {
        dark: false,
        colors: {
          ...baseLightColors,
          background: "#FFFBF2",
          "surface-variant": "#F8EED6",
          primary: "#F57C00",
          secondary: "#FFB74D",
        },
      },
      dark: { dark: true, colors: { ...darkColors } },
      "dark-blue": {
        dark: true,
        colors: {
          ...darkColors,
          background: "#0F1828",
          surface: "#17243A",
          "surface-variant": "#243653",
          primary: "#64B5F6",
          secondary: "#4DD0E1",
        },
      },
      "dark-green": {
        dark: true,
        colors: {
          ...darkColors,
          background: "#0F1D16",
          surface: "#182A20",
          "surface-variant": "#264435",
          primary: "#81C784",
          secondary: "#4DB6AC",
        },
      },
      "dark-red": {
        dark: true,
        colors: {
          ...darkColors,
          background: "#211113",
          surface: "#301A1E",
          "surface-variant": "#4A262B",
          primary: "#EF9A9A",
          secondary: "#FFAB91",
        },
      },
      "dark-purple": {
        dark: true,
        colors: {
          ...darkColors,
          background: "#1B1124",
          surface: "#291A36",
          "surface-variant": "#402953",
          primary: "#CE93D8",
          secondary: "#B39DDB",
        },
      },
    },
  },
});

performance.mark("boot:vuetify-ready");
createApp(App).use(vuetify).mount("#app");
performance.mark("boot:mounted");
