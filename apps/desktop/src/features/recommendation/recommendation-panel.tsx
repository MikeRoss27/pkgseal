import { Skeleton } from "@/components/ui/ui/skeleton"
import { PolicyConfidenceBadge } from "@/components/data-display/confidence-badge"
import { PolicyPresetSelect } from "@/components/data-display/policy-preset-select"
import { DetailRow, DetailRows } from "@/components/data-display/detail-row"
import { usePolicyPresets, usePolicyEvaluation } from "@/services/queries/policy.queries"
import type { PolicyCandidateDto, RecommendationDto } from "@/services/ipc/client"
import { ChevronRight } from "lucide-react"
import { useState, useMemo } from "react"

interface RecommendationPanelProps {
  candidates: PolicyCandidateDto[]
}

function RecommendationSkeleton() {
  return (
    <div className="cn:space-y-3">
      <Skeleton className="cn:h-9 cn:w-64" />
      <Skeleton className="cn:h-16 cn:w-full" />
      <Skeleton className="cn:h-16 cn:w-full" />
    </div>
  )
}

function EmptyRecommendation() {
  return <p className="cn:text-sm cn:text-muted-foreground cn:py-6 cn:text-center">Run policy evaluation to see a recommendation.</p>
}

interface AlternativeItemProps {
  alt: { candidate: PolicyCandidateDto; score: number; reasons: { kind: string; detail: string; contribution: number }[]; warnings: { kind: string; detail: string; severity: string; penalty: number }[] }
  isRecommended: boolean
}

function AlternativeItem({ alt, isRecommended }: AlternativeItemProps) {
  const [expanded, setExpanded] = useState(false)

  return (
    <div
      className={`cn:border-l-2 cn:border cn:border-border cn:rounded-md cn:transition-colors cn:cursor-pointer ${
        isRecommended ? "cn:border-l-[color:var(--brand-steel)]" : "cn:border-l-transparent"
      }`}
      onClick={() => setExpanded((v) => !v)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault()
          setExpanded((v) => !v)
        }
      }}
      aria-expanded={expanded}
    >
      <div className="cn:p-2.5 cn:flex cn:items-start cn:justify-between cn:gap-3">
        <div className="cn:min-w-0 cn:flex-1 cn:space-y-0.5">
          <div className="cn:flex cn:items-center cn:gap-2 cn:flex-wrap">
            <span className="cn:text-[13px] cn:font-medium cn:truncate">{alt.candidate.packageName}</span>
            <span className="cn:text-[11px] cn:text-muted-foreground">{alt.candidate.source}</span>
            <span className="cn:text-[11px] cn:text-muted-foreground cn:font-mono">{alt.candidate.version}</span>
            {isRecommended && <span className="cn:text-[11px] cn:font-medium cn:text-[color:var(--brand-steel)]">Recommended</span>}
          </div>
          <div className="cn:flex cn:items-center cn:gap-2 cn:text-[11px] cn:text-muted-foreground">
            <span className="cn:font-mono">score {String(alt.score)}</span>
            {alt.reasons.length > 0 && <span>· {alt.reasons.length} reason{alt.reasons.length !== 1 ? "s" : ""}</span>}
            {alt.warnings.length > 0 && <span>· {alt.warnings.length} warning{alt.warnings.length !== 1 ? "s" : ""}</span>}
          </div>
        </div>
        <ChevronRight className={`cn:size-3.5 cn:text-muted-foreground cn:shrink-0 cn:transition-transform ${expanded ? "cn:rotate-90" : ""}`} />
      </div>

      {expanded && (
        <div className="cn:border-t cn:border-border cn:px-2.5 cn:py-2 cn:space-y-2.5">
          {alt.reasons.length > 0 && (
            <DetailRows>
              {alt.reasons.map((r, i) => (
                <DetailRow key={i} label={r.kind} value={r.detail} hint={`contribution: ${r.contribution > 0 ? "+" : ""}${r.contribution}`} />
              ))}
            </DetailRows>
          )}
          {alt.warnings.length > 0 && (
            <DetailRows>
              {alt.warnings.map((w, i) => (
                <DetailRow key={i} label={w.kind} value={w.detail} hint={`severity: ${w.severity} · penalty: ${w.penalty}`} />
              ))}
            </DetailRows>
          )}
        </div>
      )}
    </div>
  )
}

export function RecommendationPanel({ candidates }: RecommendationPanelProps) {
  const { data: presets, isLoading: presetsLoading } = usePolicyPresets()
  const [selectedPreset, setSelectedPreset] = useState<string>("balanced")

  const effectivePreset = useMemo(() => {
    if (!presets || presets.length === 0) return selectedPreset
    if (presets.some((p) => p.id === selectedPreset)) return selectedPreset
    return presets[0]?.id ?? selectedPreset
  }, [presets, selectedPreset])

  const { data: evaluation, isLoading: evalLoading, error } = usePolicyEvaluation(effectivePreset, candidates)

  const recommendation: RecommendationDto | null = evaluation?.recommendation ?? null
  const recommendedCandidate = recommendation?.recommended ?? null
  const allAlternatives = useMemo(() => [
    ...(recommendedCandidate ? [{ candidate: recommendedCandidate, score: recommendation!.score, reasons: recommendation!.reasons, warnings: recommendation!.warnings }] : []),
    ...(recommendation?.alternatives ?? []),
  ], [recommendation, recommendedCandidate])

  const isRecommendedCandidate = (c: PolicyCandidateDto) =>
    recommendedCandidate !== null &&
    recommendedCandidate.packageName === c.packageName &&
    recommendedCandidate.source === c.source

  if (presetsLoading) {
    return <RecommendationSkeleton />
  }

  if (!presets || presets.length === 0) {
    return <EmptyRecommendation />
  }

  return (
    <div className="cn:space-y-3">
      <div className="cn:flex cn:items-center cn:justify-between cn:gap-3">
        <div className="cn:flex cn:items-center cn:gap-2">
          <PolicyConfidenceBadge confidence={recommendation?.confidence as "High" | "Medium" | "Low" | "Uncertain" | "None" ?? "None"} />
          <span className="cn:text-[11px] cn:text-muted-foreground">
            score <span className="cn:font-mono cn:text-foreground">{recommendation?.score != null ? String(recommendation.score) : "0"}</span>
          </span>
        </div>
        <PolicyPresetSelect value={effectivePreset} onChange={setSelectedPreset} presets={presets} disabled={evalLoading} className="cn:w-[180px]" />
      </div>

      {error && <p className="cn:text-[12px] cn:text-[color:var(--danger)]">Policy evaluation failed: {error.message}</p>}

      {!recommendation && !evalLoading ? (
        <EmptyRecommendation />
      ) : (
        <>
          {recommendation?.reasons.length ? (
            <DetailRows>
              {recommendation.reasons.map((r, i) => (
                <DetailRow key={i} label={r.kind} value={r.detail} hint={`contribution: ${r.contribution > 0 ? "+" : ""}${r.contribution}`} />
              ))}
            </DetailRows>
          ) : null}

          {recommendation?.warnings.length ? (
            <DetailRows>
              {recommendation.warnings.map((w, i) => (
                <DetailRow key={i} label={w.kind} value={w.detail} hint={`severity: ${w.severity} · penalty: ${w.penalty}`} />
              ))}
            </DetailRows>
          ) : null}

          {allAlternatives.length > 0 && (
            <div className="cn:space-y-1.5">
              <h4 className="cn:text-[11px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground">
                Candidates ({allAlternatives.length})
              </h4>
              <div className="cn:space-y-1.5">
                {allAlternatives.map((alt, i) => (
                  <AlternativeItem
                    key={`${alt.candidate.source}-${alt.candidate.packageName}-${i}`}
                    alt={alt}
                    isRecommended={isRecommendedCandidate(alt.candidate)}
                  />
                ))}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  )
}
