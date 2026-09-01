import { Check, Minus, AlertTriangle } from "lucide-react"

export type EvidenceState = "positive" | "neutral" | "warning"

interface EvidenceFactProps {
  label: string
  state: EvidenceState
  detail?: string
}

const ICON: Record<EvidenceState, typeof Check> = {
  positive: Check,
  neutral: Minus,
  warning: AlertTriangle,
}

const COLOR: Record<EvidenceState, string> = {
  positive: "cn:text-[color:var(--success)]",
  neutral: "cn:text-muted-foreground/50",
  warning: "cn:text-[color:var(--warning)]",
}

/**
 * A single piece of evidence, colored by what it actually means:
 * positive = verified/present, neutral = unknown/not applicable (never
 * rendered as dangerous), warning = a real risk-relevant signal.
 */
export function EvidenceFact({ label, state, detail }: EvidenceFactProps) {
  const Icon = ICON[state]
  return (
    <div className="cn:flex cn:items-start cn:gap-2 cn:py-1">
      <Icon className={`cn:size-3.5 cn:shrink-0 cn:mt-0.5 ${COLOR[state]}`} />
      <div className="cn:min-w-0">
        <p className="cn:text-[12.5px] cn:text-foreground cn:leading-snug">{label}</p>
        {detail && <p className="cn:text-[11px] cn:text-muted-foreground cn:leading-snug">{detail}</p>}
      </div>
    </div>
  )
}
