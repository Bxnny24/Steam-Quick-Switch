import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./en.json";
import de from "./de.json";

/** OS locale on first load; settings override this once loaded. */
function initialLanguage(): "en" | "de" {
  return navigator.language.toLowerCase().startsWith("de") ? "de" : "en";
}

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    de: { translation: de },
  },
  lng: initialLanguage(),
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

export default i18n;
