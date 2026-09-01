import { useEffect } from "react"
import { SourceBadge } from "@/components/data-display/source-badge"
import { ConfidenceBadge } from "@/components/data-display/confidence-badge"
import { EmptyState } from "@/components/data-display/empty-state"
import { InlineError } from "@/components/data-display/error-state"
import { Skeleton } from "@/components/ui/ui/skeleton"
import { Card } from "@/components/ui/ui/card"
import { Search, SearchX, Package } from "lucide-react"
import {
  MIN_SEARCH_QUERY_LENGTH,
  useResolvedApplications,
} from "@/services/queries/resolver.queries"
import type { ResolvedApplicationDto } from "@/services/ipc/client"

// Stable empty-array reference
const NO_APPLICATIONS: ResolvedApplicationDto[] = []

function firstDescription(app: ResolvedApplicationDto): string | null {
  return app.candidateDetails.find((d) => d.summary.description)?.summary.description ?? null
}

function ResultSkeleton() {
  return (
    <div className="cn:space-y-2" aria-hidden="true">
      {[0, 1, 2].map((i) => (
        <Card key={i} className="cn:p-3 cn:space-y-3">
          <div className="cn:flex cn:items-center cn:justify-between">
            <Skeleton className="cn:h-4 cn:w-32" />
            <Skeleton className="cn:h-4 cn:w-16 cn:rounded-full" />
          </div>
          <Skeleton className="cn:h-3 cn:w-full" />
          <div className="cn:flex cn:gap-1.5">
            <Skeleton className="cn:h-5 cn:w-16 cn:rounded-full" />
            <Skeleton className="cn:h-5 cn:w-14 cn:rounded-full" />
          </div>
        </Card>
      ))}
    </div>
  )
}

interface ResultRowProps {
  app: ResolvedApplicationDto
  selected: boolean
  onSelect: (id: string) => void
}

function ResultRow({ app, selected, onSelect }: ResultRowProps) {
  const description = firstDescription(app)
  return (
    <button
      type="button"
      onClick={() => onSelect(app.id)}
      aria-pressed={selected}
      className={`cn:group cn:flex cn:w-full cn:flex-col cn:gap-2 cn:rounded-xl cn:border cn:px-3.5 cn:py-3 cn:text-left cn:transition-all cn:duration-150 cn:focus-visible:outline-none cn:focus-visible:ring-2 cn:focus-visible:ring-ring/20 ${
        selected
          ? "cn:border-foreground/15 cn:bg-card cn:shadow-soft cn:ring-1 cn:ring-foreground/10"
          : "cn:border-border/60 cn:bg-card/50 cn:hover:bg-card cn:hover:border-border cn:hover:shadow-soft"
      }`}
    >
      <div className="cn:flex cn:items-start cn:justify-between cn:gap-2">
        <span className="cn:font-medium cn:text-[13.5px] cn:leading-tight cn:text-foreground cn:line-clamp-1">{app.displayName}</span>
        <ConfidenceBadge confidence={app.confidence} />
      </div>

      {description ? (
        <p className="cn:line-clamp-2 cn:text-xs cn:leading-relaxed cn:text-muted-foreground">{description}</p>
      ) : (
        <p className="cn:text-xs cn:text-muted-foreground/60 cn:italic">No description available</p>
      )}

      <div className="cn:flex cn:flex-wrap cn:items-center cn:gap-1.5 cn:pt-0.5">
        {app.candidates.map((candidate) => (
          <SourceBadge key={candidate.candidateId} source={candidate.source} />
        ))}
        <span className="cn:ml-auto cn:text-[11px] cn:text-muted-foreground cn:flex cn:items-center cn:gap-1">
          <Package className="cn:size-3" />
          {app.candidates.length} variant{app.candidates.length !== 1 ? "s" : ""}
        </span>
      </div>

      {app.signals.length > 0 && (
        <div className="cn:flex cn:flex-wrap cn:gap-1 cn:pt-1 cn:border-t cn:border-dashed cn:border-border/60 cn:mt-1">
          {app.signals.slice(0, 3).map((s, i) => (
            <span key={i} className="cn:inline-flex cn:items-center cn:rounded-md cn:bg-muted cn:px-1.5 cn:py-0.5 cn:text-[11px] cn:text-muted-foreground">
              {s.signalType}
            </span>
          ))}
          {app.signals.length > 3 && <span className="cn:text-[11px] cn:text-muted-foreground">+{app.signals.length - 3} more</span>}
        </div>
      )}
    </button>
  )
}

interface ResolvedApplicationsPanelProps {
  query: string
  selectedId: string | null
  onSelect: (id: string) => void
}

export function ResolvedApplicationsPanel({
  query,
  selectedId,
  onSelect,
}: ResolvedApplicationsPanelProps) {
  const { data, isLoading, isError, error, refetch, isFetching } = useResolvedApplications(query)
  const applications = data?.applications ?? NO_APPLICATIONS

  useEffect(() => {
    const firstApp = applications[0]
    if (!firstApp) return
    if (!selectedId || !applications.some((app) => app.id === selectedId)) {
      onSelect(firstApp.id)
    }
  }, [applications, selectedId, onSelect])

  if (query.trim().length < MIN_SEARCH_QUERY_LENGTH) {
    return (
      <div className="cn:rounded-xl cn:border cn:border-dashed cn:bg-muted/20 cn:px-4 cn:py-6 cn:text-center">
        <Search className="cn:size-5 cn:mx-auto cn:text-muted-foreground/60 cn:mb-2" />
        <p className="cn:text-sm cn:text-muted-foreground">Type at least {MIN_SEARCH_QUERY_LENGTH} characters to search.</p>
        <p className="cn:text-xs cn:text-muted-foreground/70 cn:mt-1">Sources are queried in parallel and results stream in.</p>
      </div>
    )
  }

  if (isLoading) {
    return <ResultSkeleton />
  }

  if (isError) {
    return (
      <div className="cn:space-y-3">
        <InlineError message={`Failed to resolve applications: ${error instanceof Error ? error.message : String(error)}`} onRetry={() => void refetch()} />
        {isFetching && <ResultSkeleton />}
      </div>
    )
  }

  if (applications.length === 0) {
    return (
      <EmptyState
        icon={<SearchX className="cn:size-5" />}
        title={`No applications found for “${query}”`}
        description="Try a different spelling, a package name, or one of the suggested examples above."
      />
    )
  }

  return (
    <div className="cn:space-y-2">
      <div className="cn:flex cn:items-center cn:gap-2 cn:text-xs cn:text-muted-foreground cn:px-1">
        <span className="cn:inline-flex cn:size-1.5 cn:rounded-full cn:bg-emerald-500 cn:animate-pulse" />
        {applications.length} application{applications.length !== 1 ? "s" : ""} resolved
        {isFetching && <span className="cn:ml-auto cn:inline-flex cn:items-center cn:gap-1"><span className="cn:size-2 cn:rounded-full cn:bg-muted-foreground/40 cn:animate-pulse" /> Updating…</span>}
      </div>
      <div className="cn:flex cn:flex-col cn:gap-2 cn:max-h-[60vh] cn:overflow-auto cn:pr-1 cn:pb-1 cn:scrollbar-thin">
        {applications.map((app) => (
          <ResultRow key={app.id} app={app} selected={app.id === selectedId} onSelect={onSelect} />
        ))}
      </div>
      <p className="cn:text-[11px] cn:text-muted-foreground cn:px-1">Confidence reflects signal agreement (name, publisher, homepage …).</p>
    </div>
  )
}
