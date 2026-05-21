import { NativeConnection, Worker } from "@temporalio/worker";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import * as apiActivities from "./activities/api";
import * as kybActivities from "./activities/kyb";
import { startBridge } from "./client";
import { TASK_QUEUE } from "./types";

const DEFAULT_TEMPORAL_ADDRESS = "localhost:7233";

async function main() {
  const connection = await NativeConnection.connect({
    address: process.env.TEMPORAL_ADDRESS ?? DEFAULT_TEMPORAL_ADDRESS,
  });

  await startBridge();

  const worker = await Worker.create({
    connection,
    namespace: process.env.TEMPORAL_NAMESPACE ?? "default",
    taskQueue: TASK_QUEUE,
    workflowsPath: join(currentDir(), "workflows/client-onboarding.ts"),
    activities: {
      ...kybActivities,
      ...apiActivities,
    },
  });

  await worker.run();
}

function currentDir() {
  return dirname(fileURLToPath(import.meta.url));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
