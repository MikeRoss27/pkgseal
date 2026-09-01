import { openUrl } from "@tauri-apps/plugin-opener"
import { Badge } from "@/components/ui/ui/badge"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/ui/tabs"
import { Separator } from "@/components/ui/ui/separator"
import { Skeleton } from "@/components/ui/ui/skeleton"
import { SourceBadge } from "@/components/data-display/source-badge"
import { ConfidenceBadge } from "@/components/data-display/confidence-badge"
import { EmptyState } from "@/components/data-display/empty-state"
import { DetailRow, DetailRows } from "@/components/data-display/detail-row"
import { TransactionPreviewPanel } from "@/features/transaction-preview/transaction-preview-panel"
import { CandidateCompare } from "@/features/resolver/candidate-compare"
import { EvidenceFact, type EvidenceState } from "@/components/data-display/evidence-fact"
import { useResolvedApplications } from "@/services/queries/resolver.queries"
import type { CandidateEvidenceDto, PackageDetailsDto, ResolvedApplicationDto } from "@/services/ipc/client"
import { ExternalLink, Search, Box, ScrollText, ChevronDown } from "lucide-react"
import { formatBytes } from "@/lib/format"
import { getPackageSourceInfo } from "@/lib/package-source"
import { RecommendationPanel } from "@/features/recommendation/recommendation-panel"
import { mapResolvedAppToPolicyCandidates, findRealEvidence } from "@/lib/policy-mapper"
import { useEffect, useMemo, useState } from "react"

function detailsFor(app: ResolvedApplicationDto, packageId: string): PackageDetailsDto | null {
  return app.candidateDetails.find((d) => d.summary.id === packageId) ?? null
}

function bestDescription(app: ResolvedApplicationDto): string | null {
  return app.candidateDetails.find((d) => d.summary.description)?.summary.description ?? null
}

function bestHomepage(app: ResolvedApplicationDto): string | null {
  return app.candidateDetails.find((d) => d.url)?.url ?? null
}

function flatpakAppIdForPreview(
  candidate: { source: string; packageName: string; packageId: string },
  details: PackageDetailsDto | null,
): string | undefined {
  if (candidate.source !== "flatpak") return undefined
  const raw = details?.rawMetadata as Record<string, unknown> | undefined
  const appIdFromMeta = raw?.["application_id"] ?? raw?.["applicationId"]
  if (typeof appIdFromMeta === "string" && appIdFromMeta.includes(".")) return appIdFromMeta
  const pid = candidate.packageId
  const slash = pid.indexOf("/")
  if (slash !== -1) {
    const suffix = pid.slice(slash + 1)
    if (suffix.includes(".")) return suffix
  }
  const sid = details?.summary.id
  if (typeof sid === "string") {
    const s = sid.indexOf("/")
    if (s !== -1) {
      const suffix = sid.slice(s + 1)
      if (suffix.includes(".")) return suffix
    }
    if (sid.includes(".")) return sid
  }
  if (candidate.packageName.includes(".")) return candidate.packageName
  return undefined
}

interface ApplicationDetailPanelProps {
  query: string
  applicationId: string | null
}

function DetailSkeleton() {
  return (
    <div className="cn:space-y-3">
      <Skeleton className="cn:h-5 cn:w-40" />
      <Skeleton className="cn:h-3 cn:w-full" />
      <Skeleton className="cn:h-3 cn:w-3/4" />
      <Skeleton className="cn:h-20 cn:w-full" />
      <Skeleton className="cn:h-20 cn:w-full" />
    </div>
  )
}

interface CandidateRowProps {
  app: ResolvedApplicationDto
  candidate: ResolvedApplicationDto["candidates"][number]
  compareChecked: boolean
  onToggleCompare: () => void
  expanded: boolean
  onToggleExpand: () => void
}

function CandidateRow({ app, candidate, compareChecked, onToggleCompare, expanded, onToggleExpand }: CandidateRowProps) {
  const details = detailsFor(app, candidate.packageId)
  return (
    <div className="cn:border cn:border-border cn:rounded-lg">
      <div className="cn:flex cn:items-start cn:gap-2.5 cn:p-2.5">
        <input
          type="checkbox"
          checked={compareChecked}
          onChange={onToggleCompare}
          aria-label={`Select ${candidate.packageName} (${candidate.source}) for comparison`}
          className="cn:mt-1 cn:size-3.5 cn:accent-[color:var(--brand-steel)]"
        />
        <div className="cn:min-w-0 cn:flex-1 cn:space-y-1">
          <div className="cn:flex cn:items-center cn:gap-2 cn:flex-wrap">
            <SourceBadge source={candidate.source} />
            <span className="cn:text-[13px] cn:font-medium cn:truncate">{candidate.packageName}</span>
            <span className="cn:text-[11px] cn:text-muted-foreground cn:font-mono cn:truncate">{candidate.packageId}</span>
            {details && <span className="cn:text-[11px] cn:font-mono cn:bg-muted cn:px-1.5 cn:py-0.5 cn:rounded">{details.summary.version}</span>}
          </div>
          {details?.summary.description && details.summary.description !== bestDescription(app) && (
            <p className="cn:text-[11.5px] cn:text-muted-foreground cn:line-clamp-1">{details.summary.description}</p>
          )}
          <div className="cn:flex cn:flex-wrap cn:items-center cn:gap-1.5 cn:text-[11px] cn:text-muted-foreground">
            {details?.summary.repository && <span>{details.summary.repository}</span>}
            {details?.architecture && <span>· {details.architecture}</span>}
            {details?.license && <span>· {details.license}</span>}
            {details && (
              <span>
                · {details.summary.installedSize ? formatBytes(details.summary.installedSize) : "size unknown"}
              </span>
            )}
          </div>
        </div>
        <div className="cn:flex cn:shrink-0 cn:items-center cn:gap-1.5">
          {details?.summary.installed ? (
            <Badge variant="secondary" className="cn:text-[11px] cn:gap-1">
              <Box className="cn:size-3" /> Installed
            </Badge>
          ) : (
            <Badge variant="outline" className="cn:text-[11px] cn:text-muted-foreground">Not installed</Badge>
          )}
          <button
            type="button"
            onClick={onToggleExpand}
            className="cn:inline-flex cn:items-center cn:gap-1 cn:rounded-md cn:border cn:border-border cn:px-2 cn:py-1 cn:text-[11px] cn:font-medium cn:text-muted-foreground cn:hover:text-foreground cn:hover:border-foreground/30 cn:transition-colors"
          >
            <ScrollText className="cn:size-3" /> Preview
            <ChevronDown className={`cn:size-3 cn:transition-transform ${expanded ? "cn:rotate-180" : ""}`} />
          </button>
        </div>
      </div>

      {expanded && (
        <div className="cn:border-t cn:border-border cn:p-2.5 cn:bg-muted/20">
          <TransactionPreviewPanel
            source={candidate.source}
            packageName={candidate.packageName}
            version={details?.summary.version ?? "unknown"}
            appId={flatpakAppIdForPreview(candidate, details)}
            reason="user requested preview"
          />
        </div>
      )}
    </div>
  )
}

function CandidateInspectorSelector({
  app,
  candidateId,
  onChange,
}: {
  app: ResolvedApplicationDto
  candidateId: string | null
  onChange: (id: string) => void
}) {
  if (app.candidates.length <= 1) return null
  return (
    <div className="cn:flex cn:items-center cn:gap-1.5 cn:pb-1">
      <span className="cn:text-[11px] cn:text-muted-foreground cn:shrink-0">Inspecting</span>
      <div className="cn:flex cn:items-center cn:gap-1">
        {app.candidates.map((c) => (
          <button
            key={c.candidateId}
            type="button"
            onClick={() => onChange(c.candidateId)}
            aria-pressed={candidateId === c.candidateId}
            className={`cn:rounded-md cn:border cn:px-1.5 cn:py-0.5 cn:text-[11px] cn:font-medium cn:transition-colors ${
              candidateId === c.candidateId
                ? "cn:border-[color:var(--brand-steel)] cn:text-foreground"
                : "cn:border-transparent cn:text-muted-foreground cn:hover:text-foreground"
            }`}
          >
            {getPackageSourceInfo(c.source).label}
          </button>
        ))}
      </div>
    </div>
  )
}

function evidenceFacts(evidence: CandidateEvidenceDto): { integrity: { label: string; state: EvidenceState; detail?: string }[]; risk: { label: string; state: EvidenceState; detail?: string }[] } {
  const elevated = (level: string) => level === "broad" || level === "excessive" || level === "host" || level === "system"
  return {
    integrity: [
      { label: "Digital signature", state: evidence.signaturePresent ? "positive" : "neutral", detail: evidence.signaturePresent ? "Present" : "Not detected" },
      { label: "Checksum", state: evidence.checksumPresent ? "positive" : "neutral", detail: evidence.checksumPresent ? (evidence.checksumValidated ? "Present and validated" : "Present") : "Not detected" },
      { label: "Sandboxed runtime", state: evidence.sandboxed ? "positive" : "neutral", detail: evidence.sandboxed ? "Runs in a sandbox" : "Runs with host access" },
      { label: "Publisher verified", state: evidence.publisherVerified ? "positive" : "neutral", detail: evidence.publisherVerified ? "Verified publisher" : "Not verified" },
    ],
    risk: [
      ...evidence.findings.map((f) => ({ label: f, state: "warning" as EvidenceState, detail: "Static PKGBUILD finding" })),
      { label: "Install script", state: evidence.installScriptPresent ? "warning" : "neutral", detail: evidence.installScriptPresent ? "Runs a post-install script" : "None" },
      { label: "Build logic changed", state: evidence.buildLogicChanged ? "warning" : "neutral", detail: evidence.buildLogicChanged ? "Build steps changed since last review" : "Unchanged" },
      { label: "Filesystem access", state: elevated(evidence.filesystemAccess) ? "warning" : "neutral", detail: evidence.filesystemAccess },
      { label: "D-Bus access", state: elevated(evidence.dbusAccess) ? "warning" : "neutral", detail: evidence.dbusAccess },
      { label: "Network access", state: evidence.networkAccess ? "warning" : "neutral", detail: evidence.networkAccess ? "Requested" : "Not requested" },
      { label: "Device access", state: evidence.deviceAccess ? "warning" : "neutral", detail: evidence.deviceAccess ? "Requested" : "Not requested" },
    ],
  }
}

export function ApplicationDetailPanel({ query, applicationId }: ApplicationDetailPanelProps) {
  const { data, isLoading } = useResolvedApplications(query)
  const app = data?.applications.find((a) => a.id === applicationId) ?? null

  const [activeTab, setActiveTab] = useState("overview")
  const [expandedCandidateId, setExpandedCandidateId] = useState<string | null>(null)
  const [compareSelection, setCompareSelection] = useState<Set<string>>(new Set())
  const [inspectCandidateId, setInspectCandidateId] = useState<string | null>(null)

  useEffect(() => {
    // eslint-disable-next-line -- intentional reset on applicationId change
    setActiveTab("overview")
    setExpandedCandidateId(null)
    setCompareSelection(new Set())
    setInspectCandidateId(null)
  }, [applicationId])

  const policyCandidates = useMemo(() => (app ? mapResolvedAppToPolicyCandidates(app) : []), [app])
  const description = app ? bestDescription(app) : null
  const homepage = app ? bestHomepage(app) : null

  const effectiveInspectCandidate = useMemo(() => {
    if (!app || app.candidates.length === 0) return null
    if (inspectCandidateId && app.candidates.some((c) => c.candidateId === inspectCandidateId)) {
      return app.candidates.find((c) => c.candidateId === inspectCandidateId) ?? null
    }
    const primary = app.candidates.find((c) => c.source === app.primarySource)
    return primary ?? app.candidates[0] ?? null
  }, [app, inspectCandidateId])

  const inspectEvidence = useMemo(() => {
    if (!app || !effectiveInspectCandidate) return null
    return findRealEvidence(app, effectiveInspectCandidate.source, effectiveInspectCandidate.packageName)
  }, [app, effectiveInspectCandidate])

  const compareEntries = useMemo(() => {
    if (!app) return []
    return app.candidates
      .filter((c) => compareSelection.has(c.candidateId))
      .map((c) => ({ candidate: c, details: detailsFor(app, c.packageId) }))
  }, [app, compareSelection])

  if (!app) {
    if (isLoading && applicationId) {
      return <DetailSkeleton />
    }
    return (
      <EmptyState
        icon={<Search className="cn:size-5" />}
        title="Select an application to see its details."
        description="Pick a result on the left to inspect its sources, provenance and recommendation."
      />
    )
  }

  if (isLoading && !data) {
    return <DetailSkeleton />
  }

  return (
    <div className="cn:space-y-3">
      <div className="cn:space-y-1.5">
        <div className="cn:flex cn:items-start cn:justify-between cn:gap-3">
          <div className="cn:min-w-0 cn:flex-1">
            <h2 className="cn:text-[15px] cn:font-semibold cn:leading-tight cn:flex cn:items-center cn:gap-2">
              <span className="cn:truncate">{app.displayName}</span>
              {app.primarySource && (
                <span className="cn:text-[11px] cn:font-normal cn:text-muted-foreground">{getPackageSourceInfo(app.primarySource).label} primary</span>
              )}
            </h2>
            {description ? (
              <p className="cn:text-[12.5px] cn:text-muted-foreground cn:leading-relaxed cn:line-clamp-2">{description}</p>
            ) : (
              <p className="cn:text-[12.5px] cn:italic cn:text-muted-foreground/60">No description available for this application.</p>
            )}
          </div>
          <ConfidenceBadge confidence={app.confidence} />
        </div>

        <div className="cn:flex cn:flex-wrap cn:items-center cn:gap-1.5 cn:text-[11px] cn:text-muted-foreground">
          <span>{app.candidates.length} source{app.candidates.length !== 1 ? "s" : ""}</span>
          <Separator orientation="vertical" className="cn:h-3" />
          <span>{app.signals.length} signal{app.signals.length !== 1 ? "s" : ""}</span>
          {app.canonicalName !== app.displayName && (
            <>
              <Separator orientation="vertical" className="cn:h-3" />
              <span className="cn:font-mono">{app.canonicalName}</span>
            </>
          )}
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="cn:gap-3">
        <TabsList variant="line" className="cn:w-full cn:justify-start cn:border-b cn:border-border cn:h-8 cn:p-0">
          <TabsTrigger value="overview" className="cn:text-[13px]">Overview</TabsTrigger>
          <TabsTrigger value="provenance" className="cn:text-[13px]">Provenance</TabsTrigger>
          <TabsTrigger value="integrity" className="cn:text-[13px]">Integrity</TabsTrigger>
          <TabsTrigger value="risk" className="cn:text-[13px]">Risk</TabsTrigger>
          <TabsTrigger value="recommendation" className="cn:text-[13px]">Recommendation</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="cn:space-y-2.5 cn:mt-0">
          {compareEntries.length >= 2 && (
            <CandidateCompare entries={compareEntries} onClose={() => setCompareSelection(new Set())} />
          )}

          <div className="cn:space-y-2">
            {app.candidates.map((candidate) => (
              <CandidateRow
                key={candidate.candidateId}
                app={app}
                candidate={candidate}
                compareChecked={compareSelection.has(candidate.candidateId)}
                onToggleCompare={() =>
                  setCompareSelection((prev) => {
                    const next = new Set(prev)
                    if (next.has(candidate.candidateId)) next.delete(candidate.candidateId)
                    else next.add(candidate.candidateId)
                    return next
                  })
                }
                expanded={expandedCandidateId === candidate.candidateId}
                onToggleExpand={() => setExpandedCandidateId((prev) => (prev === candidate.candidateId ? null : candidate.candidateId))}
              />
            ))}
          </div>

          {homepage && (
            <div className="cn:flex cn:items-center cn:justify-between cn:gap-3 cn:border cn:border-border cn:rounded-lg cn:px-3 cn:py-2">
              <div className="cn:min-w-0 cn:flex-1">
                <p className="cn:text-[10.5px] cn:font-medium cn:uppercase cn:tracking-wide cn:text-muted-foreground">Homepage</p>
                <button
                  type="button"
                  onClick={() => void openUrl(homepage)}
                  className="cn:text-[12.5px] cn:text-foreground cn:underline cn:underline-offset-2 cn:decoration-muted-foreground/40 cn:hover:decoration-foreground cn:truncate cn:block cn:w-full cn:text-left"
                >
                  {homepage}
                </button>
              </div>
              <ExternalLink className="cn:size-3.5 cn:text-muted-foreground cn:shrink-0" />
            </div>
          )}

          <p className="cn:text-[11px] cn:text-muted-foreground">
            PkgSeal never executes PKGBUILDs. Findings are static evidence that requires explanation, not proof of malware.
          </p>
        </TabsContent>

        <TabsContent value="provenance" className="cn:space-y-3 cn:mt-0">
          <div>
            <h4 className="cn:text-[11px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground cn:pb-1">Identity match signals</h4>
            {app.signals.length === 0 ? (
              <EmptyState icon={<Search className="cn:size-5" />} title="No signals" description="This application was resolved without strong signals — confidence may be lower." />
            ) : (
              <DetailRows>
                {app.signals.map((s, i) => (
                  <DetailRow key={i} label={s.signalType} value={s.value} mono />
                ))}
              </DetailRows>
            )}
          </div>

          <div>
            <CandidateInspectorSelector app={app} candidateId={effectiveInspectCandidate?.candidateId ?? null} onChange={setInspectCandidateId} />
            {inspectEvidence ? (
              <div className="cn:border cn:border-border cn:rounded-lg cn:px-3 cn:py-1 cn:divide-y cn:divide-border/60">
                <EvidenceFact label="Official repository" state={inspectEvidence.isOfficialRepository ? "positive" : "neutral"} detail={inspectEvidence.isOfficialRepository ? "Distributed by the distro's official repository" : "Not from an official repository"} />
                <EvidenceFact label="Community-maintained" state="neutral" detail={inspectEvidence.isCommunityMaintained ? "Maintained by the community, not the upstream publisher" : "Not community-maintained"} />
                <EvidenceFact label="Publisher verified" state={inspectEvidence.publisherVerified ? "positive" : "neutral"} detail={inspectEvidence.publisherVerified ? "Verified publisher" : "Not verified"} />
                <EvidenceFact label="Publisher-supported install path" state={inspectEvidence.publisherSupported ? "positive" : "neutral"} detail={inspectEvidence.publisherSupported ? "Endorsed by the upstream publisher" : "Not confirmed"} />
              </div>
            ) : (
              <p className="cn:text-[12px] cn:text-muted-foreground">No provenance evidence available for this candidate yet.</p>
            )}
          </div>

          <p className="cn:text-[11px] cn:text-muted-foreground">
            Verification status and sandbox permissions are shown separately from vulnerability data — a verified publisher is not a security guarantee.
          </p>
        </TabsContent>

        <TabsContent value="integrity" className="cn:space-y-2 cn:mt-0">
          <CandidateInspectorSelector app={app} candidateId={effectiveInspectCandidate?.candidateId ?? null} onChange={setInspectCandidateId} />
          {inspectEvidence ? (
            <div className="cn:border cn:border-border cn:rounded-lg cn:px-3 cn:py-1 cn:divide-y cn:divide-border/60">
              {evidenceFacts(inspectEvidence).integrity.map((f) => (
                <EvidenceFact key={f.label} {...f} />
              ))}
            </div>
          ) : (
            <p className="cn:text-[12px] cn:text-muted-foreground">No integrity evidence available for this candidate yet.</p>
          )}
        </TabsContent>

        <TabsContent value="risk" className="cn:space-y-2 cn:mt-0">
          <CandidateInspectorSelector app={app} candidateId={effectiveInspectCandidate?.candidateId ?? null} onChange={setInspectCandidateId} />
          {inspectEvidence ? (
            <div className="cn:border cn:border-border cn:rounded-lg cn:px-3 cn:py-1 cn:divide-y cn:divide-border/60">
              {evidenceFacts(inspectEvidence).risk.map((f, i) => (
                <EvidenceFact key={`${f.label}-${i}`} {...f} />
              ))}
            </div>
          ) : (
            <p className="cn:text-[12px] cn:text-muted-foreground">No risk evidence available for this candidate yet.</p>
          )}
          <p className="cn:text-[11px] cn:text-muted-foreground">
            A finding is static evidence that requires explanation — not proof of malicious behavior. PkgSeal never executes PKGBUILDs to produce it.
          </p>
        </TabsContent>

        <TabsContent value="recommendation" className="cn:space-y-3 cn:mt-0">
          <RecommendationPanel candidates={policyCandidates} />
          <p className="cn:text-[11px] cn:text-muted-foreground">
            Evidence → Policy → Recommendation · heuristic evidence until the inspector lands — no hard-coded <code className="cn:font-mono cn:bg-muted cn:px-1 cn:rounded">Arch &gt; Flatpak &gt; AUR</code>.
          </p>
        </TabsContent>
      </Tabs>
    </div>
  )
}
