import js from '@eslint/js'
import globals from 'globals'
import tseslint from 'typescript-eslint'
import pluginVue from 'eslint-plugin-vue'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
    {
        files: ['**/*.{js,mjs,cjs,ts,mts,cts,vue}'],
        plugins: { js },
        extends: ['js/recommended'],
        languageOptions: { globals: globals.browser },
    },
    tseslint.configs.recommended,
    pluginVue.configs['flat/essential'],
    {
        files: ['**/*.vue'],
        languageOptions: { parserOptions: { parser: tseslint.parser } },
    },
    {
        // Generated shadcn-vue primitives keep their upstream single-word names
        files: ['src/components/ui/**/*.vue'],
        rules: { 'vue/multi-word-component-names': 'off' },
    },
    globalIgnores([
        'dist/**',
        'node_modules/**',
        'src-tauri/**',
        '.vscode/**',
        '.github/**',
    ]),
])
