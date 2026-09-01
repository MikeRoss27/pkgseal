import { describe, it, expect, vi } from "vitest"
import { render, screen, waitFor } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { ResolvedApplicationsPanel } from "@/features/resolver/resolved-applications-panel"
import type { ResolveResponseDto } from "@/services/ipc/client"

vi.mock("@/services/ipc/client", () => ({
  resolveApplications: vi.fn(),
}))

const { resolveApplications } = await import("@/services/ipc/client")

function renderPanel(
  response: ResolveResponseDto,
  props: { query?: string; selectedId?: string | null; onSelect?: (id: string) => void } = {},
) {
  vi.mocked(resolveApplications).mockResolvedValue(response)
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={client}>
      <ResolvedApplicationsPanel
        query={props.query ?? "brave"}
        selectedId={props.selectedId ?? null}
        onSelect={props.onSelect ?? vi.fn()}
      />
    </QueryClientProvider>,
  )
}

const emptyResponse: ResolveResponseDto = { applications: [] }

const sampleResponse: ResolveResponseDto = {
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
        {
          candidateId: "33333333-3333-3333-3333-333333333333",
          source: "aur",
          packageName: "brave-bin",
          packageId: "aur/brave-bin",
        },
      ],
      primarySource: "arch-official",
      confidence: "Certain",
      signals: [{ signalType: "KnownAppId", value: "com.brave.browser" }],
      candidateDetails: [
        {
          summary: {
            id: "arch-official/brave-bin",
            name: "brave-bin",
            version: "1.2.3",
            description: "A privacy-focused browser",
            source: "arch-official",
            repository: "extra",
            installed: false,
            downloadSize: null,
            installedSize: null,
          },
          architecture: null,
          maintainer: null,
          url: null,
          license: null,
          dependencies: [],
          optionalDependencies: [],
          provides: [],
          conflicts: [],
          replaces: [],
          groups: [],
          buildDate: null,
          installDate: null,
          validation: null,
          rawMetadata: {},
        },
      ],
    },
  ],
}

describe("ResolvedApplicationsPanel", () => {
  it("prompts for input when the query is too short", () => {
    renderPanel(emptyResponse, { query: "b" })
    expect(screen.getByText(/Type at least 2 characters/i)).toBeInTheDocument()
    expect(resolveApplications).not.toHaveBeenCalled()
  })

  it("renders an empty state", async () => {
    renderPanel(emptyResponse)
    expect(await screen.findByText(/No applications found for/i)).toBeInTheDocument()
  })

  it("renders resolved applications with description and source badges", async () => {
    renderPanel(sampleResponse)

    expect(await screen.findByText("com.brave.Browser")).toBeInTheDocument()
    expect(screen.getByText("A privacy-focused browser")).toBeInTheDocument()
    expect(screen.getByText("Arch")).toBeInTheDocument()
    expect(screen.getByText("AUR")).toBeInTheDocument()
  })

  it("auto-selects the first result", async () => {
    const onSelect = vi.fn()
    renderPanel(sampleResponse, { onSelect })

    await waitFor(() =>
      expect(onSelect).toHaveBeenCalledWith("11111111-1111-1111-1111-111111111111"),
    )
  })

  it("calls onSelect when a result is clicked", async () => {
    const onSelect = vi.fn()
    renderPanel(sampleResponse, { selectedId: "11111111-1111-1111-1111-111111111111", onSelect })

    const row = await screen.findByText("com.brave.Browser")
    row.closest("button")?.click()

    expect(onSelect).toHaveBeenCalledWith("11111111-1111-1111-1111-111111111111")
  })

  it("shows an error message when resolution fails", async () => {
    vi.mocked(resolveApplications).mockRejectedValue(new Error("boom"))
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    })
    render(
      <QueryClientProvider client={client}>
        <ResolvedApplicationsPanel query="brave" selectedId={null} onSelect={vi.fn()} />
      </QueryClientProvider>,
    )

    await waitFor(() =>
      expect(screen.getByText(/Failed to resolve applications/i)).toBeInTheDocument(),
    )
  })
})
