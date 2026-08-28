import { Badge } from "@/components/ui/ui/badge"
import { getPackageSourceInfo } from "@/lib/package-source"

interface SourceBadgeProps {
  source: string
  suffix?: string
}

export function SourceBadge({ source, suffix }: SourceBadgeProps) {
  const info = getPackageSourceInfo(source)
  return (
    <Badge variant="outline" className="cn:gap-1.5">
      <span className={`cn:size-1.5 cn:rounded-full ${info.dotClassName}`} aria-hidden="true" />
      {info.label}
      {suffix && <span className="cn:text-muted-foreground">{suffix}</span>}
    </Badge>
  )
}
