import { useState, useRef, useEffect, useCallback } from "react"
import { ResolvedApplicationsPanel } from "@/features/resolver/resolved-applications-panel"
import { ApplicationDetailPanel } from "@/features/resolver/application-detail-panel"
import { SearchBar } from "@/features/search/search-bar"
import { useDebouncedValue } from "@/lib/use-debounced-value"
import { MIN_SEARCH_QUERY_LENGTH } from "@/services/queries/resolver.queries"
import { useResolvedApplications } from "@/services/queries/resolver.queries"
import { Badge } from "@/components/ui/ui/badge"
import { Card, CardContent } from "@/components/ui/ui/card"
import { Separator } from "@/components/ui/ui/separator"
import { Kbd } from "@/components/ui/ui/kbd"
import { Search, Sparkles, Boxes, Shield, Layers } from "lucide-react"

const SEARCH_DEBOUNCE_MS = 300

function HeroHints({ onPick }: { onPick: (q: string) => void }) {
  const examples = ["Brave", "Bitwarden", "Discord", "Visual Studio Code", "Spotify", "Steam"]
  return (
    <div className="cn:grid cn:gap-3 cn:sm:grid-cols-3 cn:text-left cn:mt-6">
      <Card className="cn:bg-card/60 cn:backdrop-blur cn:border-dashed">
        <CardContent className="cn:p-4 cn:space-y-2">
          <div className="cn:size-8 cn:rounded-lg cn:bg-foreground cn:text-background cn:grid cn:place-items-center">
            <Search className="cn:size-4" />
          </div>
          <h3 className="cn:text-sm cn:font-semibold">Search across sources</h3>
          <p className="cn:text-xs cn:text-muted-foreground cn:leading-relaxed">Arch official, AUR and Flatpak — normalized into one application view.</p>
        </CardContent>
      </Card>
      <Card className="cn:bg-card/60 cn:backdrop-blur cn:border-dashed">
        <CardContent className="cn:p-4 cn:space-y-2">
          <div className="cn:size-8 cn:rounded-lg cn:bg-muted cn:grid cn:place-items-center">
            <Layers className="cn:size-4" />
          </div>
          <h3 className="cn:text-sm cn:font-semibold">Compare candidates</h3>
          <p className="cn:text-xs cn:text-muted-foreground cn:leading-relaxed">Versions, repos, install state and publisher signals side-by-side.</p>
        </CardContent>
      </Card>
      <Card className="cn:bg-card/60 cn:backdrop-blur cn:border-dashed">
        <CardContent className="cn:p-4 cn:space-y-2">
          <div className="cn:size-8 cn:rounded-lg cn:bg-muted cn:grid cn:place-items-center">
            <Shield className="cn:size-4" />
          </div>
          <h3 className="cn:text-sm cn:font-semibold">Evidence before action</h3>
          <p className="cn:text-xs cn:text-muted-foreground cn:leading-relaxed">Every recommendation is explainable: Evidence → Policy → Recommendation.</p>
        </CardContent>
      </Card>
      <div className="cn:sm:col-span-3 cn:flex cn:flex-wrap cn:items-center cn:gap-2 cn:pt-2">
        <span className="cn:text-xs cn:text-muted-foreground cn:flex cn:items-center cn:gap-1.5">
          <Boxes className="cn:size-3.5" /> Try
        </span>
        {examples.map((ex) => (
          <button
            key={ex}
            type="button"
            onClick={() => onPick(ex)}
            className="cn:rounded-full cn:border cn:bg-card cn:px-2.5 cn:py-1 cn:text-xs cn:font-medium cn:hover:bg-accent cn:transition-colors"
          >
            {ex}
          </button>
        ))}
      </div>
    </div>
  )
}

export function DiscoverPage() {
  const [query, setQuery] = useState("")
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const debouncedQuery = useDebouncedValue(query, SEARCH_DEBOUNCE_MS)
  const hasQuery = debouncedQuery.trim().length >= MIN_SEARCH_QUERY_LENGTH
  const { isFetching } = useResolvedApplications(debouncedQuery)

  // allow AppShell/topbar to focus search: expose via window event + custom event
  const searchContainerRef = useRef<HTMLDivElement>(null)
  const focusSearch = useCallback(() => {
    const input = searchContainerRef.current?.querySelector<HTMLInputElement>('input[type="search"]')
    input?.focus()
  }, [])

  useEffect(() => {
    // expose for AppShell/Topbar imperative call — global shortcut bridge
    // @ts-expect-error attach for topbar — window augmentation for focus bridge
    window.__pkgseal_focusSearch = focusSearch
    const handler = () => focusSearch()
    window.addEventListener("pkgseal:focus-search", handler)
    return () => {
      window.removeEventListener("pkgseal:focus-search", handler)
      // @ts-expect-error cleanup global bridge
      if (window.__pkgseal_focusSearch === focusSearch) delete window.__pkgseal_focusSearch
    }
  }, [focusSearch])

  return (
    <div className="cn:space-y-6 cn:animate-in cn:fade-in cn:duration-200">
      {/* Hero */}
      <div className={hasQuery ? "cn:mx-auto cn:max-w-2xl cn:space-y-3" : "cn:mx-auto cn:max-w-2xl cn:space-y-3 cn:text-center cn:pt-6 cn:md:pt-10"}>
        <div className={hasQuery ? "cn:hidden" : "cn:space-y-3"}>
          <Badge variant="secondary" className="cn:gap-1.5 cn:rounded-full cn:px-2.5 cn:py-1 cn:text-xs cn:font-normal cn:mx-auto">
            <Sparkles className="cn:size-3" /> PkgSeal • Read-only preview
          </Badge>
          <h1 className="cn:text-[28px] cn:font-semibold cn:tracking-tight cn:leading-none cn:md:text-3xl">
            Find the right package
            <span className="cn:block cn:text-muted-foreground cn:font-normal cn:text-base cn:mt-2 cn:tracking-normal">
              One search across Arch, AUR and Flatpak — then pick the best variant with evidence.
            </span>
          </h1>
        </div>

        <div ref={searchContainerRef} className={hasQuery ? "" : "cn:pt-2"}>
          <SearchBar value={query} onValueChange={setQuery} isLoading={isFetching && hasQuery} autoFocus placeholder="Search for an application (e.g. Brave, Bitwarden, Discord)" />
          {!hasQuery && (
            <p className="cn:mt-2 cn:text-xs cn:text-muted-foreground cn:flex cn:items-center cn:justify-center cn:gap-1.5">
              Press <Kbd>/</Kbd> to focus • <Kbd>⌘K</Kbd> for commands • Esc to clear
            </p>
          )}
        </div>

        {!hasQuery && <HeroHints onPick={(q) => setQuery(q)} />}
      </div>

      {/* Results */}
      {hasQuery ? (
        <div className="cn:grid cn:gap-4 cn:lg:grid-cols-[380px_minmax(0,1fr)] cn:items-start">
          <section className="cn:min-w-0 cn:lg:sticky cn:lg:top-[4.5rem] cn:self-start">
            <div className="cn:flex cn:items-center cn:justify-between cn:mb-2">
              <h3 className="cn:text-xs cn:font-semibold cn:uppercase cn:tracking-widest cn:text-muted-foreground">Results</h3>
              <span className="cn:text-xs cn:text-muted-foreground cn:hidden cn:md:inline">Pick one → details</span>
            </div>
            <ResolvedApplicationsPanel query={debouncedQuery} selectedId={selectedId} onSelect={setSelectedId} />
          </section>

          <section className="cn:min-w-0">
            <div className="cn:flex cn:items-center cn:justify-between cn:mb-2">
              <h3 className="cn:text-xs cn:font-semibold cn:uppercase cn:tracking-widest cn:text-muted-foreground">Details</h3>
              <Badge variant="outline" className="cn:text-[11px] cn:font-normal">Resolver • Evidence → Policy</Badge>
            </div>
            <ApplicationDetailPanel query={debouncedQuery} applicationId={selectedId} />
          </section>
        </div>
      ) : (
        <div className="cn:mx-auto cn:max-w-2xl cn:pt-2">
          <Separator className="cn:my-4" />
          <div className="cn:flex cn:items-center cn:justify-between cn:text-xs cn:text-muted-foreground">
            <span>Sources are checked in parallel. A slow source won’t block others.</span>
            <span className="cn:hidden cn:sm:inline">Stale • Cached • Offline states are planned</span>
          </div>
        </div>
      )}
    </div>
  )
}
