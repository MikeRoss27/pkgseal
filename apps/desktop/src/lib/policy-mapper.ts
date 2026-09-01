import type { PolicyCandidateDto } from "@/services/ipc/client"
import type { ResolvedApplicationDto, PackageDetailsDto, CandidateEvidenceDto } from "@/services/ipc/client"

/**
 * Real, backend-computed evidence for every candidate of a resolved application.
 * The resolver command evaluates the Balanced policy for each application and
 * returns the full candidate list (recommended + alternatives), each carrying
 * the `CandidateEvidence` the backend actually built from source metadata
 * (AUR PKGBUILD findings, Flatpak manifest permissions, Arch validation/signature
 * data — see `build_candidate_evidence` in apps/desktop/src-tauri/src/dto/policy.rs).
 *
 * When present, this is authoritative and should be used instead of guessing.
 */
export function extractRealPolicyCandidates(app: ResolvedApplicationDto): PolicyCandidateDto[] | null {
  const rec = app.recommendation
  if (!rec) return null
  const all: PolicyCandidateDto[] = []
  if (rec.recommended) all.push(rec.recommended)
  for (const alt of rec.alternatives) all.push(alt.candidate)
  return all.length > 0 ? all : null
}

/** Real evidence for one candidate, keyed by source + package name, from the backend recommendation. */
export function findRealEvidence(app: ResolvedApplicationDto, source: string, packageName: string): CandidateEvidenceDto | null {
  const candidates = extractRealPolicyCandidates(app)
  if (!candidates) return null
  const match = candidates.find((c) => c.source === source && c.packageName === packageName)
  return match?.evidence ?? null
}

/**
 * Heuristic mapping from resolver's `ResolvedApplication` + `PackageDetails`
 * to policy `PolicyCandidateDto`. Used only as a fallback when the backend
 * did not attach a `recommendation` (older backend / no candidates evaluable).
 * Keep the mapping explicit and conservative: prefer false-negatives over
 * inventing trust.
 */
export function mapResolvedAppToPolicyCandidates(app: ResolvedApplicationDto): PolicyCandidateDto[] {
  const real = extractRealPolicyCandidates(app)
  if (real) return real

  const detailsById = new Map<string, PackageDetailsDto>()
  for (const d of app.candidateDetails) {
    detailsById.set(d.summary.id, d)
  }

  return app.candidates.map((c): PolicyCandidateDto => {
    const details = detailsById.get(c.packageId)
    return {
      source: normalizeSource(c.source),
      packageName: c.packageName,
      version: details?.summary.version ?? "unknown",
      evidence: buildEvidence(normalizeSource(c.source), details),
    }
  })
}

function normalizeSource(raw: string): string {
  const s = raw.toLowerCase()
  if (s === "arch" || s === "arch-official" || s === "arch_official") return "arch-official"
  if (s === "aur") return "aur"
  if (s === "flatpak" || s === "flathub") return "flatpak"
  return s
}

function buildEvidence(source: string, details: PackageDetailsDto | undefined): PolicyCandidateDto["evidence"] {
  const isArch = source === "arch-official"
  const isAur = source === "aur"
  const isFlatpak = source === "flatpak"

  // Conservative defaults — avoid inventing verified publisher or signatures.
  return {
    isOfficialRepository: isArch,
    isCommunityMaintained: isAur,
    publisherVerified: false,
    publisherSupported: false,
    signaturePresent: isArch,
    checksumPresent: !isFlatpak, // Arch/AUR carry checksums; Flatpak uses ostree (not modelled here)
    checksumValidated: false,
    sandboxed: isFlatpak,
    permissionLevel: isFlatpak ? "narrow" : "moderate",
    filesystemAccess: isFlatpak ? "limited" : "host",
    dbusAccess: isFlatpak ? "none" : "none",
    networkAccess: false,
    deviceAccess: false,
    findings: collectFindings(details),
    installScriptPresent: false,
    buildLogicChanged: false,
  }
}

function collectFindings(details: PackageDetailsDto | undefined): string[] {
  if (!details) return []
  const findings: string[] = []
  const raw = details.rawMetadata as Record<string, unknown> | undefined
  if (raw && Array.isArray(raw.findings)) {
    for (const f of raw.findings as unknown[]) {
      if (typeof f === "string" && f.trim()) findings.push(f.trim())
    }
  }
  return findings
}
