const vitePlugins = [];

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ['@vueuse/nuxt', 'nuxt-svgo', '@nuxt/eslint'],
  ssr: false,
  components: [
    '@/components',
    { path: '@/components/common', prefix: 'C' },
    { path: '@/components/animation', prefix: 'A' },
    { path: '@/components/ui', prefix: 'Ui' },
  ],
  devtools: { enabled: false },
  css: ['reset-css', '@/assets/styles/base/index.scss'],
  ignore: ['**/src-tauri/**'],
  devServer: {
    host: '0',
  },
  features: { inlineStyles: false },
  compatibilityDate: '2025-07-15',
  vite: {
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      strictPort: true,
    },
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: '@use "@/assets/styles/utils" as *;',
          silenceDeprecations: ['global-builtin', 'import'],
        },
      },
    },
    plugins: vitePlugins,
  },
  eslint: {
    config: {
      stylistic: true,
    },
  },
  svgo: { defaultImport: 'component', explicitImportsOnly: true },
});
