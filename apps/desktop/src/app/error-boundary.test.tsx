import { describe, it, expect } from "vitest"
import { render, screen } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { AppShell } from "@/components/shell/app-shell"
import { ErrorBoundary } from "@/app/error-boundary"
import type { ReactElement } from "react"

function renderWithProviders(ui: ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <ErrorBoundary>
        {ui}
      </ErrorBoundary>
    </QueryClientProvider>
  )
}

describe("AppShell", () => {
  it("renders header with title", () => {
    renderWithProviders(
      <AppShell>
        <div data-testid="content">Content</div>
      </AppShell>
    )
    expect(screen.getAllByText(/PkgSeal/)[0]).toBeInTheDocument()
  })

  it("renders version badge", () => {
    renderWithProviders(
      <AppShell>
        <div data-testid="content">Content</div>
      </AppShell>
    )
    expect(screen.getByText("v0.1.0-alpha")).toBeInTheDocument()
  })

  it("renders children in card", () => {
    renderWithProviders(
      <AppShell>
        <div data-testid="custom-content">Custom Content</div>
      </AppShell>
    )
    expect(screen.getByTestId("custom-content")).toBeInTheDocument()
  })
})

describe("ErrorBoundary", () => {
  it("catches errors in children", () => {
    const ThrowError = () => {
      throw new Error("Test error")
    }
    renderWithProviders(
      <ErrorBoundary fallback={<div data-testid="fallback">Error caught</div>}>
        <ThrowError />
      </ErrorBoundary>
    )
    expect(screen.getByTestId("fallback")).toBeInTheDocument()
  })
})