import { describe, it, expect, vi } from "vitest"
import { render, screen } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { StatusBar } from "@/components/shell/status-bar"
import type { SourceAvailabilityDto } from "@/services/ipc/client"

vi.mock("@/services/ipc/client", () => ({
  getAppHealth: vi.fn(),
  getSourceAvailability: vi.fn(),
}))

const { getSourceAvailability } = await import("@/services/ipc/client")

function renderStatusBar(data: SourceAvailabilityDto[]) {
  vi.mocked(getSourceAvailability).mockResolvedValue(data)
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={client}>
      <StatusBar />
    </QueryClientProvider>,
  )
}

describe("StatusBar", () => {
  it("renders a label per source", async () => {
    renderStatusBar([
      { source: "arch-official", available: true },
      { source: "aur", available: true },
      { source: "flatpak", available: false },
    ])

    expect(await screen.findByText("Arch")).toBeInTheDocument()
    expect(screen.getByText("AUR")).toBeInTheDocument()
    expect(screen.getByText("Flatpak")).toBeInTheDocument()
  })

  it("marks an unavailable source in its title", async () => {
    renderStatusBar([{ source: "flatpak", available: false }])

    const flatpak = await screen.findByText("Flatpak")
    expect(flatpak.closest("div")).toHaveAttribute("title", "Flatpak: unavailable")
  })
})
