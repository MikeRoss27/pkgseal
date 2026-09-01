import { useState, useRef, useEffect, useCallback, useMemo } from "react"
import { ResolvedApplicationsPanel } from "@/features/resolver/resolved-applications-panel"
import { ApplicationDetailPanel } from "@/features/resolver/application-detail-panel"
import { SearchBar } from "@/features/search/search-bar"
import { useDebouncedValue } from "@/lib/use-debounced-value"
import { MIN_SEARCH_QUERY_LENGTH } from "@/services/queries/resolver.queries"
import { useResolvedApplications } from "@/services/queries/resolver.queries"
import { getPackageSourceInfo } from "@/lib/package-source"
import { SourceInfoCard } from "@/features/search/source-info-card"

const SEARCH_DEBOUNCE_MS = 300
const EXAMPLES = ["Brave", "Bitwarden", "Discord", "Visual Studio Code", "Spotify", "Steam"]
const FILTERABLE_SOURCES = ["arch-official", "aur", "flatpak"] as const

export function DiscoverPage() {
  const [query, setQuery] = useState("")
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [activeSources, setActiveSources] = useState<Set<string>>(new Set(FILTERABLE_SOURCES))
  const debouncedQuery = useDebouncedValue(query, SEARCH_DEBOUNCE_MS)
  const hasQuery = debouncedQuery.trim().length >= MIN_SEARCH_QUERY_LENGTH
  const { data, isFetching } = useResolvedApplications(debouncedQuery)

  const searchContainerRef = useRef<HTMLDivElement>(null)
  const focusSearch = useCallback(() => {
    const input = searchContainerRef.current?.querySelector<HTMLInputElement>('input[role="searchbox"]')
    input?.focus()
  }, [])

  useEffect(() => {
    // @ts-expect-error window bridge for topbar/command-palette focus
    window.__pkgseal_focusSearch = focusSearch
    const handler = () => focusSearch()
    window.addEventListener("pkgseal:focus-search", handler)
    return () => {
      window.removeEventListener("pkgseal:focus-search", handler)
      // @ts-expect-error cleanup window bridge
      if (window.__pkgseal_focusSearch === focusSearch) delete window.__pkgseal_focusSearch
    }
  }, [focusSearch])

  const toggleSource = (source: string) => {
    setActiveSources((prev) => {
      const next = new Set(prev)
      if (next.has(source)) next.delete(source)
      else next.add(source)
      return next.size === 0 ? new Set(FILTERABLE_SOURCES) : next
    })
  }

  const totalApplications = data?.applications.length ?? 0

  const filteredCount = useMemo(() => {
    if (!data) return 0
    return data.applications.filter((app) => app.candidates.some((c) => activeSources.has(c.source))).length
  }, [data, activeSources])

  return (
    <div className="cn:flex cn:h-full cn:flex-col">
      {/* The search bar is the page's one job — give it the row to itself. */}
      <div className="cn:flex cn:shrink-0 cn:justify-center cn:border-b cn:border-border cn:px-3 cn:py-3">
        <div ref={searchContainerRef} className="cn:w-full cn:max-w-lg">
          <SearchBar value={query} onValueChange={setQuery} isLoading={isFetching && hasQuery} autoFocus placeholder="Search an application — Brave, Bitwarden, Discord…" />
        </div>
      </div>

      {/* Master–detail workspace */}
      <div className="cn:flex cn:flex-1 cn:min-h-0">
        <aside className="cn:flex cn:w-[320px] cn:shrink-0 cn:flex-col cn:border-r cn:border-border">
          {/* Filters live where they act: directly above the list they filter. */}
          <div className="cn:flex cn:shrink-0 cn:items-center cn:gap-1 cn:border-b cn:border-border cn:px-2.5 cn:py-1.5">
            <span className="cn:pr-0.5 cn:text-[10px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground/70">
              Sources
            </span>
            {FILTERABLE_SOURCES.map((source) => {
              const info = getPackageSourceInfo(source)
              const on = activeSources.has(source)
              return (
                <button
                  key={source}
                  type="button"
                  onClick={() => toggleSource(source)}
                  aria-pressed={on}
                  className={`cn:inline-flex cn:items-center cn:gap-1.5 cn:rounded-md cn:border cn:px-2 cn:py-1 cn:text-[11px] cn:font-medium cn:transition-colors ${
                    on ? "cn:border-border cn:bg-muted cn:text-foreground" : "cn:border-transparent cn:text-muted-foreground/50 cn:hover:text-muted-foreground"
                  }`}
                >
                  <span className={`cn:size-1.5 cn:rounded-full ${info.dotClassName} ${on ? "" : "cn:opacity-30"}`} aria-hidden="true" />
                  {info.label}
                </button>
              )
            })}
            <div className="cn:ml-auto">
              <SourceInfoCard />
            </div>
          </div>

          <div className="cn:flex-1 cn:overflow-y-auto">
            {hasQuery ? (
              <ResolvedApplicationsPanel query={debouncedQuery} selectedId={selectedId} onSelect={setSelectedId} activeSources={activeSources} />
            ) : (
              <div className="cn:px-3 cn:py-6 cn:space-y-5">
                <p className="cn:text-xs cn:text-muted-foreground">
                  Type at least {MIN_SEARCH_QUERY_LENGTH} characters to search Arch, AUR and Flatpak.
                </p>
                <div className="cn:space-y-2">
                  <p className="cn:text-[10px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground/70">Try one of these</p>
                  <div className="cn:flex cn:flex-wrap cn:gap-1.5">
                    {EXAMPLES.map((ex) => (
                      <button
                        key={ex}
                        type="button"
                        onClick={() => setQuery(ex)}
                        className="cn:rounded-md cn:border cn:border-border cn:bg-card cn:px-2.5 cn:py-1 cn:text-[11px] cn:font-medium cn:text-muted-foreground cn:transition-colors cn:hover:border-[color:var(--brand-steel-line)] cn:hover:text-foreground"
                      >
                        {ex}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>
        </aside>

        <section className="cn:flex-1 cn:min-w-0 cn:overflow-y-auto cn:p-3">
          <ApplicationDetailPanel query={debouncedQuery} applicationId={selectedId} />
        </section>
      </div>

      {/* Developer-tool style status line — same height as the sidebar footer so the
          border reads as one continuous strip across the window. */}
      <div className="cn:flex cn:h-6 cn:shrink-0 cn:items-center cn:border-t cn:border-border cn:bg-muted/30 cn:px-3 cn:text-[11px] cn:text-muted-foreground">
        <span>
          {hasQuery
            ? isFetching
              ? "Querying sources…"
              : `${filteredCount} of ${totalApplications} application${totalApplications !== 1 ? "s" : ""} shown`
            : "Idle"}
        </span>
      </div>
    </div>
  )
}
