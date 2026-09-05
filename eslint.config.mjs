import eslint from "@eslint/js";
import prettier from "eslint-config-prettier";
import vue from "eslint-plugin-vue";
import globals from "globals";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["**/dist/**", "**/coverage/**", "**/target/**"] },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  ...vue.configs["flat/recommended"],
  {
    files: ["apps/desktop/src/**/*.{ts,vue}"],
    languageOptions: {
      globals: {
        ...globals.browser,
        __BKMT_VERSION__: "readonly",
      },
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    files: ["**/*.test.ts", "**/tests/**/*.ts", "**/*.config.ts"],
    languageOptions: { globals: globals.node },
  },
  {
    files: ["scripts/**/*.mjs", "*.config.mjs"],
    languageOptions: { globals: globals.node },
  },
  prettier,
);
