import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Home } from "./home";

const makeApp = (id: string, displayName: string, isDefault: boolean) =>
  ({
    descriptor: { id, display_name: displayName, default: isDefault },
    installed: isDefault ? { app_id: id, version: "1.0.0", install_root: `/tmp/${id}`, last_updated: "2026-06-16" } : null,
    pending: null,
    launcher_too_old: false,
  }) as any;

describe("Home", () => {
  it("renders one tile per registered app", () => {
    const { getAllByTestId } = render(
      <Home
        apps={[
          makeApp("hermes", "Hermes Agent", true),
          makeApp("myapp", "My App", false),
        ]}
        onSelect={() => {}}
        onInstallAllPending={() => {}}
      />
    );
    expect(getAllByTestId("app-tile").length).toBe(2);
  });

  it("renders default app first", () => {
    const { getAllByTestId } = render(
      <Home
        apps={[
          makeApp("myapp", "My App", false),
          makeApp("hermes", "Hermes Agent", true),
        ]}
        onSelect={() => {}}
        onInstallAllPending={() => {}}
      />
    );
    expect(getAllByTestId("app-tile")[0].textContent).toContain("Hermes");
  });

  it("shows no apps installed when list is empty", () => {
    const { getByText } = render(
      <Home apps={[]} onSelect={() => {}} onInstallAllPending={() => {}} />
    );
    expect(getByText("No apps installed")).toBeTruthy();
  });
});