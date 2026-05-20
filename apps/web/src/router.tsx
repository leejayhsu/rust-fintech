import { createRootRoute, createRoute, createRouter, Outlet } from "@tanstack/react-router";

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

const routeTree = rootRoute.addChildren([indexRoute, dashboardRoute]);

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
