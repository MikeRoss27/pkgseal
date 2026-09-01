import { describe, it, expect, vi, beforeEach } from "vitest"
import { getAppHealth, getSourceAvailability } from "@/services/ipc/client"
import {
  appHealthQueryOptions,
  sourceAvailabilityQueryOptions,
} from "@/services/queries/system.queries"

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}))

const { invoke } = await import("@tauri-apps/api/core")

describe("IPC Client", () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  it("calls invoke with correct command and returns parsed health", async () => {
    const mockHealth = {
      app_name: "PkgSeal",
      app_version: "0.1.0",
      engine_sources: ["arch", "aur", "flatpak"],
    }
    vi.mocked(invoke).mockResolvedValue(mockHealth)

    const result = await getAppHealth()

    expect(invoke).toHaveBeenCalledWith("app_health")
    expect(result).toEqual(mockHealth)
  })

  it("propagates invoke errors", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Tauri error"))

    await expect(getAppHealth()).rejects.toThrow("Tauri error")
  })

  it("calls invoke with source_availability", async () => {
    const mockAvailability = [{ source: "arch-official", available: true }]
    vi.mocked(invoke).mockResolvedValue(mockAvailability)

    const result = await getSourceAvailability()

    expect(invoke).toHaveBeenCalledWith("source_availability")
    expect(result).toEqual(mockAvailability)
  })
})

describe("System Queries", () => {
  it("has correct query key", () => {
    expect(appHealthQueryOptions.queryKey).toEqual(["system", "health"])
  })

  it("has query function", () => {
    expect(typeof appHealthQueryOptions.queryFn).toBe("function")
  })

  it("has correct query key for source availability", () => {
    expect(sourceAvailabilityQueryOptions.queryKey).toEqual(["system", "source-availability"])
  })
})