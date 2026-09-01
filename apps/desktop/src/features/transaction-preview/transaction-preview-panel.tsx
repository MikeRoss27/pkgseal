import { Skeleton } from "@/components/ui/ui/skeleton"
import { useTransactionPreview } from "@/services/queries/transaction.queries"
import { TransactionPreviewCard } from "./transaction-preview-card"
import { AlertTriangle, Loader2 } from "lucide-react"

interface TransactionPreviewPanelProps {
  source: string
  packageName: string
  version: string
  appId?: string
  reason?: string
  enabled?: boolean
}

function PreviewSkeleton() {
  return (
    <div className="cn:space-y-2">
      <Skeleton className="cn:h-4 cn:w-40" />
      <Skeleton className="cn:h-16 cn:w-full" />
    </div>
  )
}

export function TransactionPreviewPanel({ source, packageName, version, appId, reason, enabled = true }: TransactionPreviewPanelProps) {
  const hasInput = Boolean(source && packageName && version)
  const shouldFetch = enabled && hasInput

  const { data, isLoading, error, isFetching } = useTransactionPreview({
    source,
    packageName,
    version,
    appId,
    reason,
  })

  if (!hasInput || !shouldFetch) {
    return <p className="cn:text-[12px] cn:text-muted-foreground">No candidate selected for preview.</p>
  }

  if (isLoading) {
    return <PreviewSkeleton />
  }

  if (error) {
    const message = error instanceof Error ? error.message : String(error)
    const isTauriMissing = message.includes("Tauri runtime unavailable")
    return (
      <div className="cn:space-y-1.5">
        <p className="cn:flex cn:items-start cn:gap-1.5 cn:text-[12px] cn:text-[color:var(--danger)]">
          <AlertTriangle className="cn:size-3.5 cn:shrink-0 cn:mt-0.5" /> {message}
        </p>
        {isTauriMissing && (
          <p className="cn:text-[11px] cn:text-muted-foreground">
            Run <code className="cn:font-mono cn:bg-muted cn:px-1 cn:rounded">bun run tauri dev</code> to enable native preview.
          </p>
        )}
      </div>
    )
  }

  if (!data) {
    return <p className="cn:text-[12px] cn:text-muted-foreground">No plan could be generated for {packageName} ({source}).</p>
  }

  return (
    <div className="cn:space-y-2">
      {isFetching && (
        <div className="cn:flex cn:items-center cn:gap-1.5 cn:text-[11px] cn:text-muted-foreground">
          <Loader2 className="cn:size-3 cn:animate-spin" /> Updating preview…
        </div>
      )}
      <TransactionPreviewCard plan={data.plan} preview={data.preview} />
    </div>
  )
}
