import {
  Client,
  Connection,
  WorkflowExecutionAlreadyStartedError,
} from "@temporalio/client";
import express from "express";
import { z } from "zod";

import { clientOnboardingWorkflow, reviewSignal } from "./workflows/client-onboarding";
import { TASK_QUEUE } from "./types";

const DEFAULT_TEMPORAL_ADDRESS = "localhost:7233";
const DEFAULT_TEMPORAL_NAMESPACE = "default";
const DEFAULT_BRIDGE_PORT = 4100;

const startSchema = z.object({
  onboardingId: z.string().min(1),
  workflowId: z.string().min(1),
});

const reviewSchema = z.object({
  approved: z.boolean(),
  note: z.string().nullable().optional(),
});

export async function startBridge() {
  const app = express();
  const temporal = await getTemporalClient();

  app.use(express.json());

  app.post("/internal/workflows/client-onboarding/start", async (req, res) => {
    const parsed = startSchema.safeParse(req.body);
    if (!parsed.success) {
      res.status(422).json({ error: "invalid start payload" });
      return;
    }

    try {
      await temporal.workflow.start(clientOnboardingWorkflow, {
        taskQueue: TASK_QUEUE,
        workflowId: parsed.data.workflowId,
        args: [
          {
            onboardingId: parsed.data.onboardingId,
            workflowId: parsed.data.workflowId,
          },
        ],
      });
      res.json({});
    } catch (error) {
      if (error instanceof WorkflowExecutionAlreadyStartedError) {
        res.json({});
        return;
      }
      console.error("failed to start client onboarding workflow", error);
      res.status(502).json({ error: "failed to start workflow" });
    }
  });

  app.post(
    "/internal/workflows/client-onboarding/:onboardingId/review",
    async (req, res) => {
      const parsed = reviewSchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(422).json({ error: "invalid review payload" });
        return;
      }

      const workflowId = `client-onboarding-${req.params.onboardingId}`;
      try {
        const handle = temporal.workflow.getHandle(workflowId);
        await handle.signal(reviewSignal, parsed.data);
        res.json({});
      } catch (error) {
        if (isDuplicateSafeSignalError(error)) {
          res.json({});
          return;
        }
        console.error("failed to signal client onboarding workflow", error);
        res.status(502).json({ error: "failed to signal workflow" });
      }
    },
  );

  const port = Number(process.env.ONBOARDING_WORKER_PORT ?? DEFAULT_BRIDGE_PORT);
  return app.listen(port, () => {
    console.log(`Temporal onboarding bridge listening on ${port}`);
  });
}

async function getTemporalClient() {
  const connection = await Connection.connect({
    address: process.env.TEMPORAL_ADDRESS ?? DEFAULT_TEMPORAL_ADDRESS,
  });

  return new Client({
    connection,
    namespace: process.env.TEMPORAL_NAMESPACE ?? DEFAULT_TEMPORAL_NAMESPACE,
  });
}

function isDuplicateSafeSignalError(error: unknown) {
  const message = error instanceof Error ? error.message.toLowerCase() : String(error);
  return (
    message.includes("not found") ||
    message.includes("closed") ||
    message.includes("completed") ||
    message.includes("terminated") ||
    message.includes("canceled") ||
    message.includes("cancelled")
  );
}

if (import.meta.url === `file://${process.argv[1]}`) {
  await startBridge();
}
