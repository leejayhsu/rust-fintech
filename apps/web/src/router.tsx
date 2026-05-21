import { createRootRoute, createRoute, createRouter, Outlet } from "@tanstack/react-router";

import { AdminOnboardingRoute } from "./routes/admin-onboarding";
import { ClientOnboardingRoute } from "./routes/client-onboarding";
import { DashboardRoute } from "./routes/dashboard";
import { LandingRoute } from "./routes/landing";

const rootRoute = createRootRoute({
  component: RootLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: LandingRoute,
});

const dashboardRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/dashboard",
  component: DashboardRoute,
});

const clientOnboardingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/onboarding/client",
  component: ClientOnboardingRoute,
});

const adminOnboardingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/onboarding",
  component: AdminOnboardingRoute,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  dashboardRoute,
  clientOnboardingRoute,
  adminOnboardingRoute,
]);

export const router = createRouter({
  routeTree,
  defaultPreload: "intent",
});

function RootLayout() {
  return (
    <main className="app-shell">
      <Outlet />
    </main>
  );
}

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
