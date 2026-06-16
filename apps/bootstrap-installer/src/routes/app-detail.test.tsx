import { describe, it, expect, vi, beforeEach } from "vitest";
import { render } from "@testing-library/react";
import { AppDetail } from "./app-detail";
import { $bootstrapByApp } from "../store";

vi.mock("@nanostores/react", () => ({
  useStore: vi.fn(),
}));

describe("AppDetail", () => {
  beforeEach(() => {
    $bootstrapByApp.set({});
  });

  it("Launch enabled when installed", () => {
    const app = {
      descriptor: { id: "x", display_name: "X", uninstall_supported: true, app_settings_url: null },
      installed: { app_id: "x", version: "1.0.0", install_root: "/x", last_updated: "2026-01-01T00:00:00Z" },
      pending: null,
      launcher_too_old: false,
    } as any;

    const { getByText } = render(
      <AppDetail app={app} onAction={() => {}} />
    );
    expect(getByText("Launch").closest("button")).not.toBeDisabled();
  });

  it("Install disabled when launcher too old", () => {
    const app = {
      descriptor: { id: "x", display_name: "X", uninstall_supported: true, app_settings_url: null },
      installed: null,
      pending: null,
      launcher_too_old: true,
    } as any;

    const { getByText } = render(
      <AppDetail app={app} onAction={() => {}} />
    );
    expect(getByText("Install").closest("button")).toBeDisabled();
  });
});