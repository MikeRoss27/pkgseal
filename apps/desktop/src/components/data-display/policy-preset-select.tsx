import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/ui/select"
import { usePolicyPresets } from "@/services/queries/policy.queries"
import type { PresetDto } from "@/services/ipc/client"

interface PolicyPresetSelectProps {
  value: string
  onChange: (value: string) => void
  disabled?: boolean
  className?: string
  /** Optional override — when provided, avoids internal query. */
  presets?: PresetDto[]
}

export function PolicyPresetSelect({ value, onChange, disabled, className, presets: presetsProp }: PolicyPresetSelectProps) {
  const { data: fetched } = usePolicyPresets()
  const presets = presetsProp ?? fetched

  if (!presets) return null

  return (
    <Select
      value={value}
      onValueChange={(v) => {
        if (typeof v === "string" && v.length > 0) onChange(v)
      }}
      disabled={disabled}
    >
      <SelectTrigger className={`cn:w-[220px] ${className ?? ""}`}>
        <SelectValue placeholder="Select policy preset" />
      </SelectTrigger>
      <SelectContent>
        {presets.map((preset: PresetDto) => (
          <SelectItem key={preset.id} value={preset.id}>
            <div className="cn:flex cn:flex-col cn:gap-0.5">
              <span className="cn:font-medium cn:text-sm">{preset.id.replace("-", " ")}</span>
              <span className="cn:text-xs cn:text-muted-foreground">{preset.description}</span>
            </div>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}