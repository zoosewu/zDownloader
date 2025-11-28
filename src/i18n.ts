// i18n configuration
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import zhTW from './locales/zh-TW.json';
import en from './locales/en.json';
import ja from './locales/ja.json';

i18n
  .use(initReactI18next)
  .init({
    resources: {
      'zh-TW': { translation: zhTW },
      'en': { translation: en },
      'ja': { translation: ja },
    },
    lng: 'zh-TW', // default language
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false, // React already escapes by default
    },
  });

export default i18n;
