import { useQuery } from "@tanstack/react-query"
import { Info } from "lucide-react"
import { PreviewCard, PreviewCardContent, PreviewCardTrigger } from "@/components/ui/ui/preview-card"
import { sourceAvailabilityQueryOptions } from "@/services/queries/system.queries"
import { getPackageSourceInfo } from "@/lib/package-source"

const SOURCE_NOTES: Record<string, string> = {
  "arch-official": "Official repository — signed packages, checksums validated.",
  aur: "Community-maintained — PKGBUILDs are statically inspected, never executed.",
  flatpak: "Sandboxed runtime — filesystem, network and D-Bus permissions are surfaced.",
}

/** Compact info affordance next to search: hover to see what each source means and whether it's reachable. */
export function SourceInfoCard() {
  const { data } = useQuery(sourceAvailabilityQueryOptions)
  const availability = new Map((data ?? []).map((s) => [s.source, s.available]))

  return (
    <PreviewCard>
      <PreviewCardTrigger
        render={
          <button
            type="button"
            aria-label="About package sources"
            className="cn:inline-flex cn:size-6 cn:items-center cn:justify-center cn:rounded-md cn:text-muted-foreground cn:hover:bg-muted cn:hover:text-foreground cn:transition-colors"
          />
        }
      >
        <Info className="cn:size-3.5" />
      </PreviewCardTrigger>
      <PreviewCardContent>
        <p className="cn:text-[11px] cn:font-semibold cn:uppercase cn:tracking-wide cn:text-muted-foreground cn:pb-2">Package sources</p>
        <div className="cn:space-y-2.5">
          {(["arch-official", "aur", "flatpak"] as const).map((source) => {
            const info = getPackageSourceInfo(source)
            const available = availability.get(source)
            return (
              <div key={source} className="cn:flex cn:items-start cn:gap-2">
                <span className={`cn:size-1.5 cn:mt-1.5 cn:rounded-full cn:shrink-0 ${info.dotClassName}`} aria-hidden="true" />
                <div className="cn:min-w-0">
                  <p className="cn:text-[12.5px] cn:font-medium cn:text-foreground">
                    {info.label}
                    {available === false && <span className="cn:ml-1.5 cn:text-[11px] cn:font-normal cn:text-muted-foreground">(unreachable)</span>}
                  </p>
                  <p className="cn:text-[11px] cn:text-muted-foreground cn:leading-snug">{SOURCE_NOTES[source]}</p>
                </div>
              </div>
            )
          })}
        </div>
      </PreviewCardContent>
    </PreviewCard>
  )
}
