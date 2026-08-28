import { describe, it, expect, vi } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { ApplicationDetailPanel } from "@/features/resolver/application-detail-panel"
import type { ResolveResponseDto } from "@/services/ipc/client"

vi.mock("@/services/ipc/client", () => ({
  resolveApplications: vi.fn(),
}))

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}))

const { resolveApplications } = await import("@/services/ipc/client")
const { openUrl } = await import("@tauri-apps/plugin-opener")

const response: ResolveResponseDto = {
  applications: [
    {
      id: "app-1",
      canonicalName: "brave",
      displayName: "com.brave.Browser",
      candidates: [
        {
          candidateId: "c-1",
          source: "arch-official",
          packageName: "brave-bin",
          packageId: "arch-official/brave-bin",
        },
      ],
      primarySource: "arch-official",
      confidence: "Certain",
      signals: [],
      candidateDetails: [
        {
          summary: {
            id: "arch-official/brave-bin",
            name: "brave-bin",
            version: "1.2.3",
            description: "A privacy-focused browser",
            source: "arch-official",
            repository: "extra",
            installed: true,
            downloadSize: null,
            installedSize: null,
          },
          architecture: null,
          maintainer: null,
          url: "https://brave.com",
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

function renderDetail(applicationId: string | null) {
  vi.mocked(resolveApplications).mockResolvedValue(response)
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <ApplicationDetailPanel query="brave" applicationId={applicationId} />
    </QueryClientProvider>,
  )
}

describe("ApplicationDetailPanel", () => {
  it("prompts for a selection when nothing is selected", () => {
    renderDetail(null)
    expect(screen.getByText(/Select an application/i)).toBeInTheDocument()
  })

  it("renders the selected application's sources, version, and homepage", async () => {
    renderDetail("app-1")

    expect(await screen.findByText("com.brave.Browser")).toBeInTheDocument()
    expect(screen.getByText("A privacy-focused browser")).toBeInTheDocument()
    expect(screen.getByText("brave-bin")).toBeInTheDocument()
    expect(screen.getByText("1.2.3")).toBeInTheDocument()
    expect(screen.getByText("Installed")).toBeInTheDocument()
    expect(screen.getByText("https://brave.com")).toBeInTheDocument()
  })

  it("opens the homepage via the opener plugin when clicked", async () => {
    renderDetail("app-1")

    fireEvent.click(await screen.findByText("https://brave.com"))

    expect(openUrl).toHaveBeenCalledWith("https://brave.com")
  })
})
