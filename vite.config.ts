import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: ["apps/web/dist/**"],
    plugins: ["typescript"],
    overrides: [
      {
        files: ["apps/web/**"],
        plugins: ["typescript", "react"],
      },
    ],
  },
  fmt: {
    ignorePatterns: ["apps/web/dist/**"],
    semi: true,
    singleQuote: false,
    sortPackageJson: true,
  },
});
