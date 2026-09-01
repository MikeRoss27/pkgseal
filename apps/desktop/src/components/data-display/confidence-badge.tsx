import { Stamp } from "lucide-react"
import { Badge } from "@/components/ui/ui/badge"
import type { MatchConfidenceLevel } from "@/services/ipc/client"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/ui/tooltip"

const CONFIDENCE_META: Record<MatchConfidenceLevel, { label: string; variant: "secondary" | "outline" | "default"; dot: string; hint: string }> = {
  Certain: { label: "Certain", variant: "secondary", dot: "cn:bg-emerald-500", hint: "High signal agreement across sources" },
  High: { label: "High", variant: "secondary", dot: "cn:bg-emerald-500", hint: "Strong match" },
  Medium: { label: "Medium", variant: "outline", dot: "cn:bg-amber-500", hint: "Moderate confidence — verify details" },
  Low: { label: "Possible", variant: "outline", dot: "cn:bg-amber-500", hint: "Low confidence — possible match, check signals" },
  Speculative: { label: "Speculative", variant: "outline", dot: "cn:bg-zinc-400", hint: "Speculative — weak signals" },
}

interface ConfidenceBadgeProps {
  confidence: MatchConfidenceLevel
  compact?: boolean
}

/** Certain matches get PkgSeal's namesake mark — an actual stamp, not a status dot. */
function ConfidenceMark({ confidence, dot }: { confidence: MatchConfidenceLevel; dot: string }) {
  if (confidence === "Certain") {
    return <Stamp className="cn:size-2.5 cn:text-[color:var(--brand-steel)]" aria-hidden="true" />
  }
  return <span className={`cn:size-1.5 cn:rounded-full ${dot}`} aria-hidden="true" />
}

export function ConfidenceBadge({ confidence, compact }: ConfidenceBadgeProps) {
  const meta = CONFIDENCE_META[confidence] ?? CONFIDENCE_META.Speculative
  return (
    <Tooltip>
      <TooltipTrigger render={<Badge variant={meta.variant} className="cn:gap-1 cn:font-normal cn:text-[11px]" />}>
        <ConfidenceMark confidence={confidence} dot={meta.dot} />
        {compact ? meta.label.slice(0, 3) : meta.label}
      </TooltipTrigger>
      <TooltipContent>{meta.hint}</TooltipContent>
    </Tooltip>
  )
}

export type PolicyConfidence = "High" | "Medium" | "Low" | "Uncertain" | "None"

const POLICY_CONFIDENCE_META: Record<PolicyConfidence, { label: string; variant: "secondary" | "outline" | "default"; dot: string; hint: string }> = {
  High: { label: "High", variant: "secondary", dot: "cn:bg-emerald-500", hint: "Strong evidence alignment with policy" },
  Medium: { label: "Medium", variant: "outline", dot: "cn:bg-amber-500", hint: "Moderate confidence — some evidence gaps" },
  Low: { label: "Low", variant: "outline", dot: "cn:bg-amber-500", hint: "Low confidence — significant evidence gaps" },
  Uncertain: { label: "Uncertain", variant: "outline", dot: "cn:bg-zinc-400", hint: "Insufficient evidence to form recommendation" },
  None: { label: "None", variant: "outline", dot: "cn:bg-zinc-400", hint: "No recommendation available" },
}

interface PolicyConfidenceBadgeProps {
  confidence: PolicyConfidence
  compact?: boolean
}

export function PolicyConfidenceBadge({ confidence, compact }: PolicyConfidenceBadgeProps) {
  const meta = POLICY_CONFIDENCE_META[confidence] ?? POLICY_CONFIDENCE_META.None
  return (
    <Tooltip>
      <TooltipTrigger render={<Badge variant={meta.variant} className="cn:gap-1 cn:font-normal cn:text-[11px]" />}>
        <span className={`cn:size-1.5 cn:rounded-full ${meta.dot}`} aria-hidden="true" />
        {compact ? meta.label.slice(0, 3) : meta.label}
      </TooltipTrigger>
      <TooltipContent>{meta.hint}</TooltipContent>
    </Tooltip>
  )
}
