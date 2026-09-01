import { useEffect, useMemo } from "react"
import { SourceBadge } from "@/components/data-display/source-badge"
import { ConfidenceBadge } from "@/components/data-display/confidence-badge"
import { EmptyState } from "@/components/data-display/empty-state"
import { InlineError } from "@/components/data-display/error-state"
import { Skeleton } from "@/components/ui/ui/skeleton"
import { SearchX } from "lucide-react"
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

/** Collapse repeated candidates from the same source into one badge with a count. */
function groupBySource(candidates: ResolvedApplicationDto["candidates"]): { source: string; count: number }[] {
  const counts = new Map<string, number>()
  for (const c of candidates) counts.set(c.source, (counts.get(c.source) ?? 0) + 1)
  return Array.from(counts, ([source, count]) => ({ source, count }))
}

function ResultSkeleton() {
  return (
    <div className="cn:divide-y cn:divide-border" aria-hidden="true">
      {[0, 1, 2].map((i) => (
        <div key={i} className="cn:px-3 cn:py-2.5 cn:space-y-2">
          <div className="cn:flex cn:items-center cn:justify-between">
            <Skeleton className="cn:h-3.5 cn:w-32" />
            <Skeleton className="cn:h-3.5 cn:w-14 cn:rounded-full" />
          </div>
          <Skeleton className="cn:h-3 cn:w-full" />
        </div>
      ))}
    </div>
  )
}

interface ResultRowProps {
  app: ResolvedApplicationDto
  selected: boolean
  onSelect: (id: string) => void
  dimmed: boolean
}

function ResultRow({ app, selected, onSelect, dimmed }: ResultRowProps) {
  const description = firstDescription(app)
  return (
    <button
      type="button"
      onClick={() => onSelect(app.id)}
      aria-pressed={selected}
      className={`cn:group cn:flex cn:w-full cn:flex-col cn:gap-1 cn:border-l-2 cn:px-3 cn:py-2 cn:text-left cn:transition-colors cn:focus-visible:outline-none cn:focus-visible:bg-muted/60 ${
        selected
          ? "cn:border-l-[color:var(--brand-steel)] cn:bg-muted/50"
          : "cn:border-l-transparent cn:hover:bg-muted/30"
      } ${dimmed ? "cn:opacity-40" : ""}`}
    >
      <div className="cn:flex cn:items-start cn:justify-between cn:gap-2">
        <span className="cn:font-medium cn:text-[13px] cn:leading-tight cn:text-foreground cn:truncate">{app.displayName}</span>
        <ConfidenceBadge confidence={app.confidence} />
      </div>

      {description ? (
        <p className="cn:line-clamp-1 cn:text-[11.5px] cn:leading-snug cn:text-muted-foreground">{description}</p>
      ) : (
        <p className="cn:text-[11.5px] cn:text-muted-foreground/50 cn:italic">No description available</p>
      )}

      <div className="cn:flex cn:flex-wrap cn:items-center cn:gap-1.5 cn:pt-0.5">
        {groupBySource(app.candidates).map(({ source, count }) => (
          <SourceBadge key={source} source={source} suffix={count > 1 ? `×${count}` : undefined} />
        ))}
      </div>
    </button>
  )
}

interface ResolvedApplicationsPanelProps {
  query: string
  selectedId: string | null
  onSelect: (id: string) => void
  /** Client-side source filter — dims (does not hide) applications with no matching candidate. */
  activeSources?: Set<string>
}

export function ResolvedApplicationsPanel({
  query,
  selectedId,
  onSelect,
  activeSources,
}: ResolvedApplicationsPanelProps) {
  const { data, isLoading, isError, error, refetch, isFetching } = useResolvedApplications(query)
  const applications = data?.applications ?? NO_APPLICATIONS

  const matchesFilter = useMemo(
    () => (app: ResolvedApplicationDto) => !activeSources || app.candidates.some((c) => activeSources.has(c.source)),
    [activeSources],
  )

  useEffect(() => {
    const firstApp = applications[0]
    if (!firstApp) return
    if (!selectedId || !applications.some((app) => app.id === selectedId)) {
      onSelect(firstApp.id)
    }
  }, [applications, selectedId, onSelect])

  if (query.trim().length < MIN_SEARCH_QUERY_LENGTH) {
    return (
      <div className="cn:px-3 cn:py-6">
        <p className="cn:text-sm cn:text-muted-foreground">Type at least {MIN_SEARCH_QUERY_LENGTH} characters to search.</p>
      </div>
    )
  }

  if (isLoading) {
    return <ResultSkeleton />
  }

  if (isError) {
    return (
      <div className="cn:p-3 cn:space-y-3">
        <InlineError message={`Failed to resolve applications: ${error instanceof Error ? error.message : String(error)}`} onRetry={() => void refetch()} />
        {isFetching && <ResultSkeleton />}
      </div>
    )
  }

  if (applications.length === 0) {
    return (
      <div className="cn:p-3">
        <EmptyState
          icon={<SearchX className="cn:size-5" />}
          title={`No applications found for “${query}”`}
          description="Try a different spelling or package name."
        />
      </div>
    )
  }

  return (
    <div className="cn:divide-y cn:divide-border">
      {applications.map((app) => (
        <ResultRow key={app.id} app={app} selected={app.id === selectedId} onSelect={onSelect} dimmed={!matchesFilter(app)} />
      ))}
    </div>
  )
}
