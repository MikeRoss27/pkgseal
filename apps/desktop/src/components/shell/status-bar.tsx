import { useQuery } from "@tanstack/react-query"
import { sourceAvailabilityQueryOptions } from "@/services/queries/system.queries"
import { getPackageSourceInfo } from "@/lib/package-source"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/ui/tooltip"

export function StatusBar() {
  const { data, isError } = useQuery(sourceAvailabilityQueryOptions)

  if (!data) {
    return (
      <span className="cn:hidden cn:md:inline-flex cn:items-center cn:gap-1.5 cn:text-[11px] cn:text-muted-foreground/60" aria-label="Source availability">
        Checking sources…
      </span>
    )
  }

  if (data.length === 0) {
    // Browser-only fallback from safeInvoke — don't show empty row, show subtle offline hint
    // or nothing to avoid layout shift; keep accessible label for tests that mock data.
    if (isError) {
      return (
        <span className="cn:inline-flex cn:items-center cn:gap-1.5 cn:text-[11px] cn:text-muted-foreground/60" title="Source availability unavailable">
          Offline
        </span>
      )
    }
    return null
  }

  return (
    <div className="cn:flex cn:items-center cn:gap-2.5" aria-label="Source availability">
      {data.map(({ source, available }) => {
        const info = getPackageSourceInfo(source)
        const title = `${info.label}: ${available ? "available" : "unavailable"}`
        return (
          <Tooltip key={source}>
            <TooltipTrigger
              render={<div title={title} className="cn:flex cn:items-center cn:gap-1.5 cn:text-[11px] cn:text-muted-foreground cn:hover:text-foreground cn:transition-colors" />}
            >
              <span
                className={`cn:size-1.5 cn:rounded-full ${available ? "cn:bg-[color:var(--success)]" : "cn:bg-muted-foreground/40"}`}
                aria-hidden="true"
              />
              <span>{info.label}</span>
            </TooltipTrigger>
            <TooltipContent>
              {title}
              {!available && " — results may be incomplete"}
            </TooltipContent>
          </Tooltip>
        )
      })}
    </div>
  )
}
