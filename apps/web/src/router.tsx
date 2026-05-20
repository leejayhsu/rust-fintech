import { createRootRoute, createRoute, createRouter, Outlet } from "@tanstack/react-router";

import { DashboardRoute } from "./routes/dashboard";

const rootRoute = createRootRoute({
  component: RootLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: DashboardRoute,
});

const routeTree = rootRoute.addChildren([indexRoute]);

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
