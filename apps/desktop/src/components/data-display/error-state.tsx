import { Alert, AlertDescription, AlertTitle } from "@/components/ui/ui/alert"
import { Button } from "@/components/ui/ui/button"
import { AlertTriangle, RefreshCw } from "lucide-react"

interface ErrorStateProps {
  title?: string
  message: string
  onRetry?: () => void
}

export function ErrorState({ title = "Something went wrong", message, onRetry }: ErrorStateProps) {
  return (
    <Alert variant="destructive" className="cn:bg-destructive/5">
      <AlertTriangle className="cn:size-4" />
      <AlertTitle>{title}</AlertTitle>
      <AlertDescription className="cn:flex cn:flex-col cn:gap-3">
        <span className="cn:break-words">{message}</span>
        {onRetry && (
          <Button variant="outline" size="sm" onClick={onRetry} className="cn:w-fit cn:gap-1.5">
            <RefreshCw className="cn:size-3.5" /> Retry
          </Button>
        )}
      </AlertDescription>
    </Alert>
  )
}

export function InlineError({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="cn:flex cn:items-center cn:justify-between cn:gap-3 cn:rounded-lg cn:border cn:border-destructive/20 cn:bg-destructive/5 cn:px-3 cn:py-2.5 cn:text-sm">
      <span className="cn:text-destructive cn:break-words">{message}</span>
      {onRetry && (
        <Button variant="ghost" size="sm" onClick={onRetry} className="cn:shrink-0 cn:h-7">
          <RefreshCw className="cn:size-3.5" /> Retry
        </Button>
      )}
    </div>
  )
}
