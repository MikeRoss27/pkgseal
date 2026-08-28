import { describe, it, expect, beforeEach } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { ThemeToggle } from "@/components/shell/theme-toggle"

describe("ThemeToggle", () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.classList.remove("dark")
  })

  it("defaults to light and shows a button to switch to dark", () => {
    render(<ThemeToggle />)
    expect(screen.getByRole("button", { name: /switch to dark theme/i })).toBeInTheDocument()
    expect(document.documentElement.classList.contains("dark")).toBe(false)
  })

  it("toggles to dark, applies the class, and persists the choice", () => {
    render(<ThemeToggle />)

    fireEvent.click(screen.getByRole("button", { name: /switch to dark theme/i }))

    expect(document.documentElement.classList.contains("dark")).toBe(true)
    expect(localStorage.getItem("pkgseal-theme")).toBe("dark")
    expect(screen.getByRole("button", { name: /switch to light theme/i })).toBeInTheDocument()
  })

  it("respects a previously persisted dark preference on mount", () => {
    localStorage.setItem("pkgseal-theme", "dark")

    render(<ThemeToggle />)

    expect(document.documentElement.classList.contains("dark")).toBe(true)
    expect(screen.getByRole("button", { name: /switch to light theme/i })).toBeInTheDocument()
  })
})
