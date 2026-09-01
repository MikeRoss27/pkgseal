import { describe, it, expect, vi, beforeEach } from "vitest"
import { resolveApplications } from "@/services/ipc/client"
import {
  resolvedApplicationsQueryOptions,
  useResolvedApplications,
} from "@/services/queries/resolver.queries"

const SAMPLE_QUERY = "brave"

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}))

const { invoke } = await import("@tauri-apps/api/core")

const mockResponse = {
  applications: [
    {
      id: "11111111-1111-1111-1111-111111111111",
      canonicalName: "brave",
      displayName: "com.brave.Browser",
      candidates: [
        {
          candidateId: "22222222-2222-2222-2222-222222222222",
          source: "arch-official",
          packageName: "brave-bin",
          packageId: "arch-official/brave-bin",
        },
      ],
      primarySource: "arch-official",
      confidence: "Certain",
      signals: [{ signalType: "KnownAppId", value: "com.brave.browser" }],
      candidateDetails: [],
    },
  ],
}

describe("Resolver IPC Client", () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  it("calls invoke with resolve_applications_command", async () => {
    vi.mocked(invoke).mockResolvedValue(mockResponse)

    const result = await resolveApplications(SAMPLE_QUERY)

    expect(invoke).toHaveBeenCalledWith("resolve_applications_command", {
      request: { query: SAMPLE_QUERY },
    })
    expect(result).toEqual(mockResponse)
  })

  it("propagates invoke errors", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Tauri error"))

    await expect(resolveApplications(SAMPLE_QUERY)).rejects.toThrow("Tauri error")
  })
})

describe("Resolver Queries", () => {
  it("has correct query key", () => {
    expect(resolvedApplicationsQueryOptions(SAMPLE_QUERY).queryKey).toEqual([
      "resolver",
      "applications",
      SAMPLE_QUERY,
    ])
  })

  it("disables the query below the minimum length", () => {
    expect(resolvedApplicationsQueryOptions("b").enabled).toBe(false)
    expect(resolvedApplicationsQueryOptions(SAMPLE_QUERY).enabled).toBe(true)
  })

  it("has query function", () => {
    expect(typeof resolvedApplicationsQueryOptions(SAMPLE_QUERY).queryFn).toBe("function")
  })

  it("exposes a useResolvedApplications hook", () => {
    expect(typeof useResolvedApplications).toBe("function")
  })
})
