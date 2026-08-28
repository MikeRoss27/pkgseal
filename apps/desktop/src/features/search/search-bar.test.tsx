import { describe, it, expect, vi } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { SearchBar } from "@/features/search/search-bar"

describe("SearchBar", () => {
  it("renders the current value", () => {
    render(<SearchBar value="brave" onValueChange={vi.fn()} />)
    expect(screen.getByRole("searchbox")).toHaveValue("brave")
  })

  it("calls onValueChange as the user types", () => {
    const onValueChange = vi.fn()
    render(<SearchBar value="" onValueChange={onValueChange} />)

    fireEvent.change(screen.getByRole("searchbox"), { target: { value: "bit" } })

    expect(onValueChange).toHaveBeenCalledWith("bit")
  })

  it("hides the clear button when empty", () => {
    render(<SearchBar value="" onValueChange={vi.fn()} />)
    expect(screen.queryByRole("button", { name: /clear search/i })).not.toBeInTheDocument()
  })

  it("clears the value when the clear button is clicked", () => {
    const onValueChange = vi.fn()
    render(<SearchBar value="brave" onValueChange={onValueChange} />)

    fireEvent.click(screen.getByRole("button", { name: /clear search/i }))

    expect(onValueChange).toHaveBeenCalledWith("")
  })
})
