/**
 * Shared display vocabulary for package sources (`PackageSource::as_str()`
 * on the Rust side). Kept in one place so every component that renders a
 * source badge agrees on the same label/color.
 */
export interface PackageSourceInfo {
  label: string
  dotClassName: string
}

const SOURCE_INFO: Record<string, PackageSourceInfo> = {
  "arch-official": { label: "Arch", dotClassName: "cn:bg-sky-500" },
  aur: { label: "AUR", dotClassName: "cn:bg-violet-500" },
  flatpak: { label: "Flatpak", dotClassName: "cn:bg-blue-500" },
}

const FALLBACK_INFO: PackageSourceInfo = {
  label: "Unknown",
  dotClassName: "cn:bg-muted-foreground",
}

export function getPackageSourceInfo(source: string): PackageSourceInfo {
  return SOURCE_INFO[source] ?? FALLBACK_INFO
}
