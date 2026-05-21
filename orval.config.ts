import { defineConfig } from "orval";

export default defineConfig({
  apiClient: {
    input: "docs/openapi.yaml",
    output: {
      target: "packages/api-client/src/generated/api.ts",
      schemas: "packages/api-client/src/generated/model",
      client: "react-query",
      httpClient: "fetch",
      mode: "split",
      clean: true,
      override: {
        mutator: {
          path: "packages/api-client/src/mutator.ts",
          name: "apiMutator",
        },
      },
    },
  },
});
