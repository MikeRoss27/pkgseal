import { openUrl } from "@tauri-apps/plugin-opener"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/ui/card"
import { Badge } from "@/components/ui/ui/badge"
import { Button } from "@/components/ui/ui/button"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/ui/tabs"
import { Separator } from "@/components/ui/ui/separator"
import { Skeleton } from "@/components/ui/ui/skeleton"
import { SourceBadge } from "@/components/data-display/source-badge"
import { ConfidenceBadge } from "@/components/data-display/confidence-badge"
import { EmptyState } from "@/components/data-display/empty-state"
import { DetailRow, DetailRows, DetailSection } from "@/components/data-display/detail-row"
import { useResolvedApplications } from "@/services/queries/resolver.queries"
import type { PackageDetailsDto, ResolvedApplicationDto } from "@/services/ipc/client"
import { ExternalLink, Package, Shield, Info, Layers, Sparkles, Search, Box } from "lucide-react"
import { formatBytes } from "@/lib/format"
import { getPackageSourceInfo } from "@/lib/package-source"

function detailsFor(app: ResolvedApplicationDto, packageId: string): PackageDetailsDto | null {
  return app.candidateDetails.find((d) => d.summary.id === packageId) ?? null
}

function bestDescription(app: ResolvedApplicationDto): string | null {
  return app.candidateDetails.find((d) => d.summary.description)?.summary.description ?? null
}

function bestHomepage(app: ResolvedApplicationDto): string | null {
  return app.candidateDetails.find((d) => d.url)?.url ?? null
}

interface ApplicationDetailPanelProps {
  query: string
  applicationId: string | null
}

function DetailSkeleton() {
  return (
    <Card>
      <CardHeader className="cn:space-y-3">
        <Skeleton className="cn:h-5 cn:w-40" />
        <Skeleton className="cn:h-3 cn:w-full" />
        <Skeleton className="cn:h-3 cn:w-3/4" />
      </CardHeader>
      <CardContent className="cn:space-y-3">
        <Skeleton className="cn:h-20 cn:w-full" />
        <Skeleton className="cn:h-20 cn:w-full" />
      </CardContent>
    </Card>
  )
}

export function ApplicationDetailPanel({ query, applicationId }: ApplicationDetailPanelProps) {
  const { data, isLoading } = useResolvedApplications(query)
  const app = data?.applications.find((a) => a.id === applicationId) ?? null

  if (!app) {
    // While loading and nothing selected yet, show prompt instead of skeleton to preserve UX and test contract
    if (isLoading && applicationId) {
      return <DetailSkeleton />
    }
    return (
      <Card className="cn:border-dashed cn:bg-muted/20">
        <CardContent className="cn:py-12">
          <EmptyState
            icon={<Search className="cn:size-5" />}
            title="Select an application to see its details."
            description="Pick a result on the left to compare its sources, versions and signals. No data is hidden — everything is explainable."
          />
        </CardContent>
      </Card>
    )
  }

  if (isLoading && !data) {
    return <DetailSkeleton />
  }

  const description = bestDescription(app)
  const homepage = bestHomepage(app)

  return (
    <Card className="cn:overflow-hidden cn:shadow-soft">
      <CardHeader className="cn:pb-3 cn:space-y-3">
        <div className="cn:flex cn:items-start cn:justify-between cn:gap-3">
          <div className="cn:min-w-0 cn:flex-1 cn:space-y-1">
            <CardTitle className="cn:text-[17px] cn:leading-tight cn:flex cn:items-center cn:gap-2">
              <span className="cn:truncate">{app.displayName}</span>
              {app.primarySource && <Badge variant="secondary" className="cn:text-[11px] cn:shrink-0">{getPackageSourceInfo(app.primarySource).label} primary</Badge>}
            </CardTitle>
            {description ? (
              <CardDescription className="cn:text-sm cn:leading-relaxed cn:line-clamp-3">{description}</CardDescription>
            ) : (
              <CardDescription className="cn:italic">No description available for this application.</CardDescription>
            )}
          </div>
          <ConfidenceBadge confidence={app.confidence} />
        </div>

        <div className="cn:flex cn:flex-wrap cn:items-center cn:gap-1.5">
          <span className="cn:text-xs cn:text-muted-foreground cn:flex cn:items-center cn:gap-1">
            <Sparkles className="cn:size-3" /> {app.candidates.length} source{app.candidates.length !== 1 ? "s" : ""}
          </span>
          <Separator orientation="vertical" className="cn:h-3 cn:mx-1" />
          <span className="cn:text-xs cn:text-muted-foreground">{app.signals.length} signal{app.signals.length !== 1 ? "s" : ""}</span>
          {app.canonicalName !== app.displayName && (
            <>
              <Separator orientation="vertical" className="cn:h-3 cn:mx-1" />
              <span className="cn:text-xs cn:font-mono cn:text-muted-foreground">{app.canonicalName}</span>
            </>
          )}
        </div>
      </CardHeader>

      <CardContent className="cn:pt-0">
        <Tabs defaultValue="sources" className="cn:gap-4">
          <TabsList className="cn:w-full cn:justify-start cn:rounded-lg cn:bg-muted/60 cn:p-1 cn:h-9">
            <TabsTrigger value="sources" className="cn:gap-1.5 cn:flex-1 cn:md:flex-initial">
              <Layers className="cn:size-3.5" /> Sources
            </TabsTrigger>
            <TabsTrigger value="signals" className="cn:gap-1.5 cn:flex-1 cn:md:flex-initial">
              <Shield className="cn:size-3.5" /> Signals
            </TabsTrigger>
            <TabsTrigger value="details" className="cn:gap-1.5 cn:flex-1 cn:md:flex-initial">
              <Info className="cn:size-3.5" /> Metadata
            </TabsTrigger>
          </TabsList>

          <TabsContent value="sources" className="cn:space-y-3 cn:mt-1">
            <div className="cn:space-y-2.5">
              {app.candidates.map((candidate) => {
                const details = detailsFor(app, candidate.packageId)
                return (
                  <div
                    key={candidate.candidateId}
                    className="cn:group cn:rounded-xl cn:border cn:border-border/80 cn:bg-card cn:p-3 cn:transition-colors cn:hover:border-border cn:hover:shadow-soft"
                  >
                    <div className="cn:flex cn:items-start cn:justify-between cn:gap-3">
                      <div className="cn:min-w-0 cn:flex-1 cn:space-y-1.5">
                        <div className="cn:flex cn:items-center cn:gap-2 cn:flex-wrap">
                          <SourceBadge source={candidate.source} />
                          <span className="cn:text-sm cn:font-medium cn:truncate">{candidate.packageName}</span>
                          <span className="cn:text-xs cn:text-muted-foreground cn:font-mono cn:truncate">· {candidate.packageId}</span>
                        </div>
                        {details?.summary.description && details.summary.description !== description && (
                          <p className="cn:text-xs cn:text-muted-foreground cn:line-clamp-2">{details.summary.description}</p>
                        )}
                        <div className="cn:flex cn:flex-wrap cn:items-center cn:gap-1.5 cn:pt-1">
                          {details?.summary.repository && <Badge variant="outline" className="cn:text-[11px] cn:font-normal">{details.summary.repository}</Badge>}
                          {details?.architecture && <Badge variant="outline" className="cn:text-[11px] cn:font-normal">{details.architecture}</Badge>}
                          {details?.license && <Badge variant="outline" className="cn:text-[11px] cn:font-normal">{details.license}</Badge>}
                        </div>
                      </div>
                      <div className="cn:flex cn:shrink-0 cn:flex-col cn:items-end cn:gap-1.5">
                        {details && (
                          <span className="cn:text-xs cn:font-mono cn:bg-muted cn:px-1.5 cn:py-0.5 cn:rounded-md">{details.summary.version}</span>
                        )}
                        {details?.summary.installed ? (
                          <Badge variant="secondary" className="cn:text-xs cn:gap-1">
                            <Box className="cn:size-3" /> Installed
                          </Badge>
                        ) : (
                          <Badge variant="outline" className="cn:text-xs cn:text-muted-foreground">Not installed</Badge>
                        )}
                      </div>
                    </div>

                    {details && (
                      <div className="cn:mt-2.5 cn:grid cn:grid-cols-2 cn:gap-2 cn:rounded-lg cn:bg-muted/40 cn:p-2.5 cn:text-xs">
                        <span className="cn:text-muted-foreground">
                          Size: <span className="cn:text-foreground cn:font-medium">{details.summary.installedSize ? formatBytes(details.summary.installedSize) : "—"}</span>
                          {details.summary.downloadSize ? <span className="cn:text-muted-foreground"> / dl {formatBytes(details.summary.downloadSize)}</span> : null}
                        </span>
                        <span className="cn:text-muted-foreground cn:text-right cn:truncate">
                          {details.maintainer ? <>Maintainer: <span className="cn:text-foreground">{details.maintainer}</span></> : <span className="cn:opacity-60">No maintainer</span>}
                        </span>
                      </div>
                    )}
                  </div>
                )
              })}
            </div>

            {homepage && (
              <div className="cn:rounded-xl cn:border cn:bg-muted/20 cn:px-3 cn:py-2.5 cn:flex cn:items-center cn:justify-between cn:gap-3">
                <div className="cn:min-w-0 cn:flex-1">
                  <p className="cn:text-xs cn:font-medium cn:uppercase cn:tracking-wide cn:text-muted-foreground">Homepage</p>
                  <button
                    type="button"
                    onClick={() => void openUrl(homepage)}
                    className="cn:text-sm cn:text-primary cn:break-all cn:text-left cn:underline cn:underline-offset-2 hover:cn:no-underline cn:truncate cn:block cn:w-full"
                  >
                    {homepage}
                  </button>
                </div>
                <Button variant="outline" size="sm" className="cn:shrink-0 cn:gap-1.5" onClick={() => void openUrl(homepage)}>
                  Open <ExternalLink className="cn:size-3.5" />
                </Button>
              </div>
            )}

            <p className="cn:text-[11px] cn:text-muted-foreground cn:px-1">
              PkgSeal never executes PKGBUILDs. Findings are static evidence that requires explanation, not proof of malware.
            </p>
          </TabsContent>

          <TabsContent value="signals" className="cn:space-y-3 cn:mt-1">
            {app.signals.length === 0 ? (
              <EmptyState icon={<Shield className="cn:size-5" />} title="No signals" description="This application was resolved without strong signals. Confidence may be lower — check sources manually." />
            ) : (
              <DetailSection title="Match signals" action={<Badge variant="outline" className="cn:text-[11px]">{app.confidence}</Badge>}>
                <DetailRows>
                  {app.signals.map((s, i) => (
                    <DetailRow key={i} label={s.signalType} value={s.value} mono />
                  ))}
                </DetailRows>
              </DetailSection>
            )}
            <div className="cn:rounded-lg cn:border cn:border-amber-200 cn:bg-amber-50 cn:dark:bg-amber-950/20 cn:dark:border-amber-900 cn:px-3 cn:py-2.5 cn:flex cn:gap-2.5">
              <Shield className="cn:size-4 cn:text-amber-600 cn:dark:text-amber-400 cn:shrink-0 cn:mt-0.5" />
              <div className="cn:space-y-1">
                <p className="cn:text-xs cn:font-semibold cn:text-amber-900 cn:dark:text-amber-100">Publisher ≠ security guarantee</p>
                <p className="cn:text-xs cn:leading-relaxed cn:text-amber-800 cn:dark:text-amber-200/80">Verification status and sandbox permissions are shown separately from vulnerability data. Always review evidence before installing.</p>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="details" className="cn:space-y-3 cn:mt-1">
            {app.candidateDetails.length === 0 ? (
              <EmptyState icon={<Package className="cn:size-5" />} title="No metadata" description="No package details were returned for this query." />
            ) : (
              app.candidateDetails.map((d) => (
                <DetailSection key={d.summary.id} title={`${d.summary.name} • ${d.summary.source}`}>
                  <DetailRows>
                    <DetailRow label="Version" value={d.summary.version} mono />
                    <DetailRow label="Repository" value={d.summary.repository ?? "—"} />
                    <DetailRow label="Arch" value={d.architecture ?? "—"} mono />
                    <DetailRow label="License" value={d.license ?? "—"} />
                    <DetailRow label="Maintainer" value={d.maintainer ?? "—"} />
                    <DetailRow label="URL" value={d.url ?? "—"} hint={d.url ?? undefined} />
                    <DetailRow label="Installed size" value={d.summary.installedSize ? formatBytes(d.summary.installedSize) : "—"} />
                    <DetailRow label="Download size" value={d.summary.downloadSize ? formatBytes(d.summary.downloadSize) : "—"} />
                    <DetailRow label="Build date" value={d.buildDate ?? "—"} />
                    {d.dependencies.length > 0 && <DetailRow label="Depends" value={d.dependencies.join(", ")} mono />}
                    {d.conflicts.length > 0 && <DetailRow label="Conflicts" value={d.conflicts.join(", ")} mono />}
                    {d.provides.length > 0 && <DetailRow label="Provides" value={d.provides.join(", ")} mono />}
                  </DetailRows>
                  {d.validation && (
                    <>
                      <Separator className="cn:my-2" />
                      <p className="cn:text-xs cn:text-muted-foreground">Validation: {d.validation}</p>
                    </>
                  )}
                </DetailSection>
              ))
            )}
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}
