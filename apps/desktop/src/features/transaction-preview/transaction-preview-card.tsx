import { Badge } from "@/components/ui/ui/badge"
import { SourceBadge } from "@/components/data-display/source-badge"
import { DetailRow, DetailRows } from "@/components/data-display/detail-row"
import type { TransactionPlanDto } from "@/services/ipc/client"
import { formatBytes, formatDate } from "@/lib/format"
import { Lock, Unlock, ScrollText, AlertTriangle } from "lucide-react"

interface TransactionPreviewCardProps {
  plan: TransactionPlanDto
  preview: string
}

function PrivilegesBadge({ required }: { required: boolean }) {
  return required ? (
    <span className="cn:inline-flex cn:items-center cn:gap-1 cn:text-[11px] cn:text-[color:var(--warning)]">
      <Lock className="cn:size-3" /> Privileges required
    </span>
  ) : (
    <span className="cn:inline-flex cn:items-center cn:gap-1 cn:text-[11px] cn:text-muted-foreground">
      <Unlock className="cn:size-3" /> No privileges
    </span>
  )
}

export function TransactionPreviewCard({ plan, preview }: TransactionPreviewCardProps) {
  const isArch = plan.source.toLowerCase() === "arch-official"
  const dl = plan.expectedDownloadSize != null ? formatBytes(plan.expectedDownloadSize) : "—"
  const disk = plan.expectedDiskChange != null ? formatBytes(Math.abs(plan.expectedDiskChange)) : "—"
  const diskSign = plan.expectedDiskChange != null && plan.expectedDiskChange < 0 ? "−" : plan.expectedDiskChange != null && plan.expectedDiskChange > 0 ? "+" : ""

  return (
    <div className="cn:space-y-2.5">
      <div className="cn:flex cn:items-center cn:justify-between cn:gap-3 cn:flex-wrap">
        <div className="cn:flex cn:items-center cn:gap-2 cn:min-w-0">
          <SourceBadge source={plan.source} />
          <span className="cn:text-[12.5px] cn:font-medium cn:truncate">{plan.summary}</span>
        </div>
        <div className="cn:flex cn:items-center cn:gap-2.5 cn:shrink-0">
          <PrivilegesBadge required={plan.privilegesRequired} />
          <Badge variant="outline" className="cn:text-[11px] cn:capitalize">{plan.state}</Badge>
        </div>
      </div>

      <DetailRows>
        <DetailRow label="Package" value={`${plan.packageName} ${plan.packageVersion}`} mono />
        <DetailRow label="Created" value={formatDate(plan.createdAt)} />
        <DetailRow label="Download" value={dl} mono />
        <DetailRow label="Disk change" value={plan.expectedDiskChange != null ? `${diskSign}${disk}` : "—"} mono />
      </DetailRows>

      <div className="cn:space-y-1.5">
        <h5 className="cn:text-[11px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground">Operations ({plan.operations.length})</h5>
        <div className="cn:space-y-1">
          {plan.operations.map((op, idx) => (
            <div key={idx} className="cn:flex cn:items-start cn:justify-between cn:gap-3 cn:border cn:border-border cn:rounded-md cn:px-2.5 cn:py-1.5">
              <div className="cn:min-w-0 cn:flex-1 cn:space-y-0.5">
                <p className="cn:text-[12px] cn:font-medium cn:leading-tight">{op.summary}</p>
                <p className="cn:text-[10.5px] cn:font-mono cn:text-muted-foreground">{op.kind}</p>
              </div>
              <span className="cn:text-[11px] cn:text-muted-foreground cn:shrink-0 cn:flex cn:items-center cn:gap-1">
                {op.requiresPrivileges ? <Lock className="cn:size-3" /> : <Unlock className="cn:size-3" />}
                {op.requiresPrivileges ? "priv" : "user"}
              </span>
            </div>
          ))}
        </div>
      </div>

      {isArch && plan.privilegesRequired && (
        <p className="cn:flex cn:items-start cn:gap-1.5 cn:text-[11px] cn:text-[color:var(--warning)]">
          <AlertTriangle className="cn:size-3.5 cn:shrink-0 cn:mt-0.5" />
          Privileged operation, not executed — PkgSeal will ask for confirmation and Polkit authorization before any system change.
        </p>
      )}

      <div className="cn:space-y-1">
        <h5 className="cn:text-[11px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground cn:flex cn:items-center cn:gap-1">
          <ScrollText className="cn:size-3" /> Inspectable preview
        </h5>
        <pre className="cn:overflow-auto cn:rounded-md cn:bg-muted/60 cn:p-2.5 cn:text-[11px] cn:leading-relaxed cn:font-mono cn:whitespace-pre-wrap cn:break-words cn:border cn:border-border">
          {preview}
        </pre>
      </div>
    </div>
  )
}
