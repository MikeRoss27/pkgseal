import { SourceBadge } from "@/components/data-display/source-badge"
import { formatBytes } from "@/lib/format"
import type { CandidateRefDto, PackageDetailsDto } from "@/services/ipc/client"

interface CompareEntry {
  candidate: CandidateRefDto
  details: PackageDetailsDto | null
}

interface CandidateCompareProps {
  entries: CompareEntry[]
  onClose: () => void
}

interface Row {
  label: string
  values: (string | null)[]
}

function buildRows(entries: CompareEntry[]): Row[] {
  const get = (fn: (e: CompareEntry) => string | null) => entries.map(fn)
  return [
    { label: "Version", values: get((e) => e.details?.summary.version ?? "unknown") },
    { label: "Repository", values: get((e) => e.details?.summary.repository ?? null) },
    { label: "Architecture", values: get((e) => e.details?.architecture ?? null) },
    { label: "License", values: get((e) => e.details?.license ?? null) },
    { label: "Maintainer", values: get((e) => e.details?.maintainer ?? null) },
    { label: "Installed", values: get((e) => (e.details?.summary.installed ? "Yes" : "No")) },
    { label: "Download size", values: get((e) => (e.details?.summary.downloadSize != null ? formatBytes(e.details.summary.downloadSize) : null)) },
    { label: "Installed size", values: get((e) => (e.details?.summary.installedSize != null ? formatBytes(e.details.summary.installedSize) : null)) },
    { label: "Build date", values: get((e) => e.details?.buildDate ?? null) },
    { label: "Dependencies", values: get((e) => (e.details ? String(e.details.dependencies.length) : null)) },
  ]
}

function allEqual(values: (string | null)[]): boolean {
  return values.every((v) => v === values[0])
}

export function CandidateCompare({ entries, onClose }: CandidateCompareProps) {
  const rows = buildRows(entries)

  return (
    <div className="cn:border cn:border-border cn:rounded-lg cn:overflow-hidden">
      <div className="cn:flex cn:items-center cn:justify-between cn:gap-2 cn:border-b cn:border-border cn:bg-muted/40 cn:px-3 cn:py-1.5">
        <span className="cn:text-[11px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground">
          Comparing {entries.length} candidates
        </span>
        <button type="button" onClick={onClose} className="cn:text-[11px] cn:text-muted-foreground cn:hover:text-foreground cn:transition-colors">
          Close
        </button>
      </div>

      <div className="cn:overflow-x-auto">
        <table className="cn:w-full cn:text-[12px] cn:border-collapse">
          <thead>
            <tr className="cn:border-b cn:border-border">
              <th className="cn:text-left cn:font-medium cn:text-muted-foreground cn:px-3 cn:py-1.5 cn:whitespace-nowrap">Attribute</th>
              {entries.map((e) => (
                <th key={e.candidate.candidateId} className="cn:text-left cn:font-medium cn:px-3 cn:py-1.5 cn:whitespace-nowrap">
                  <div className="cn:flex cn:items-center cn:gap-1.5">
                    <SourceBadge source={e.candidate.source} />
                    <span className="cn:truncate cn:max-w-[10rem]">{e.candidate.packageName}</span>
                  </div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => {
              const identical = allEqual(row.values)
              return (
                <tr key={row.label} className="cn:border-b cn:border-border/60 cn:last:border-0">
                  <td className={`cn:px-3 cn:py-1.5 cn:whitespace-nowrap cn:align-top ${identical ? "cn:text-muted-foreground/60" : "cn:text-muted-foreground cn:font-medium"}`}>
                    {row.label}
                  </td>
                  {row.values.map((v, i) => (
                    <td
                      key={i}
                      className={`cn:px-3 cn:py-1.5 cn:align-top cn:font-mono cn:text-[11.5px] ${
                        identical ? "cn:text-muted-foreground/60" : "cn:text-foreground cn:bg-[color:var(--brand-steel-tint)]"
                      }`}
                    >
                      {v ?? "—"}
                    </td>
                  ))}
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}
