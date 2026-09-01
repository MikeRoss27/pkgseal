import { Component, type ErrorInfo, type ReactNode } from "react"

interface Props {
  children: ReactNode
  fallback?: ReactNode
}

interface State {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error("ErrorBoundary caught:", error, errorInfo)
  }

  render(): ReactNode {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }
      return (
        <div role="alert" className="cn:flex cn:min-h-screen cn:items-center cn:justify-center cn:p-6 cn:bg-background">
          <div className="cn:w-full cn:max-w-md cn:rounded-2xl cn:border cn:bg-card cn:p-8 cn:text-center cn:shadow-soft">
            <div className="cn:mx-auto cn:mb-4 cn:size-10 cn:rounded-full cn:bg-destructive/10 cn:grid cn:place-items-center cn:text-destructive">
              <span aria-hidden="true">!</span>
            </div>
            <h1 className="cn:text-lg cn:font-semibold cn:tracking-tight cn:text-foreground">Something went wrong</h1>
            <p className="cn:mt-2 cn:text-sm cn:leading-relaxed cn:text-muted-foreground">
              {this.state.error?.message ?? "An unexpected error occurred"}
            </p>
            <button
              type="button"
              onClick={() => this.setState({ hasError: false, error: null })}
              className="cn:mt-6 cn:inline-flex cn:h-8 cn:items-center cn:justify-center cn:rounded-lg cn:bg-primary cn:px-4 cn:text-sm cn:font-medium cn:text-primary-foreground cn:hover:bg-primary/90"
            >
              Try again
            </button>
          </div>
        </div>
      )
    }
    return this.props.children
  }
}