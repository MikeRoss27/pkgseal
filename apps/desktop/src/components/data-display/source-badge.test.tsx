import { describe, it, expect } from "vitest"
import { render, screen } from "@testing-library/react"
import { SourceBadge } from "@/components/data-display/source-badge"

describe("SourceBadge", () => {
  it("renders the label for a known source", () => {
    render(<SourceBadge source="arch-official" />)
    expect(screen.getByText("Arch")).toBeInTheDocument()
  })

  it("renders AUR and Flatpak labels", () => {
    render(
      <>
        <SourceBadge source="aur" />
        <SourceBadge source="flatpak" />
      </>,
    )
    expect(screen.getByText("AUR")).toBeInTheDocument()
    expect(screen.getByText("Flatpak")).toBeInTheDocument()
  })

  it("falls back gracefully for an unknown source", () => {
    render(<SourceBadge source="something-new" />)
    expect(screen.getByText("Unknown")).toBeInTheDocument()
  })
})
