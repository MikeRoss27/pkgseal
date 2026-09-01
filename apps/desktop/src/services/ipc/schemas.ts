import { z } from "zod"

// ── primitives ───────────────────────────────────────────────────────

export const MatchConfidenceLevelSchema = z.enum([
  "Certain",
  "High",
  "Medium",
  "Low",
  "Speculative",
])
export type MatchConfidenceLevel = z.infer<typeof MatchConfidenceLevelSchema>

export const MatchSignalTypeSchema = z.enum([
  "KnownAppId",
  "ReverseDomainId",
  "Homepage",
  "SourceRepository",
  "Publisher",
  "DesktopFileId",
  "BinaryName",
  "ProductName",
  "FuzzyName",
])
export type MatchSignalType = z.infer<typeof MatchSignalTypeSchema>

// ── DTOs ─────────────────────────────────────────────────────────────

export const SignalSchema = z.object({
  signalType: MatchSignalTypeSchema,
  value: z.string(),
})
export type SignalDto = z.infer<typeof SignalSchema>

export const CandidateRefSchema = z.object({
  candidateId: z.string(),
  source: z.string(),
  packageName: z.string(),
  packageId: z.string(),
})
export type CandidateRefDto = z.infer<typeof CandidateRefSchema>

export const PackageSummarySchema = z.object({
  id: z.string(),
  name: z.string(),
  version: z.string(),
  description: z.string().nullable(),
  source: z.string(),
  repository: z.string().nullable(),
  installed: z.boolean(),
  downloadSize: z.number().nullable(),
  installedSize: z.number().nullable(),
})
export type PackageSummaryDto = z.infer<typeof PackageSummarySchema>

export const PackageDetailsSchema = z.object({
  summary: PackageSummarySchema,
  architecture: z.string().nullable(),
  maintainer: z.string().nullable(),
  url: z.string().nullable(),
  license: z.string().nullable(),
  dependencies: z.array(z.string()),
  optionalDependencies: z.array(z.string()),
  provides: z.array(z.string()),
  conflicts: z.array(z.string()),
  replaces: z.array(z.string()),
  groups: z.array(z.string()),
  buildDate: z.string().nullable(),
  installDate: z.string().nullable(),
  validation: z.string().nullable(),
  rawMetadata: z.record(z.string(), z.unknown()),
})
export type PackageDetailsDto = z.infer<typeof PackageDetailsSchema>

export const ResolvedApplicationSchema = z.object({
  id: z.string(),
  canonicalName: z.string(),
  displayName: z.string(),
  candidates: z.array(CandidateRefSchema),
  primarySource: z.string().nullable(),
  confidence: MatchConfidenceLevelSchema,
  signals: z.array(SignalSchema),
  candidateDetails: z.array(PackageDetailsSchema),
})
export type ResolvedApplicationDto = z.infer<typeof ResolvedApplicationSchema>

export const ResolveResponseSchema = z.object({
  applications: z.array(ResolvedApplicationSchema),
})
export type ResolveResponseDto = z.infer<typeof ResolveResponseSchema>

// ── health / source availability ─────────────────────────────────────

export const AppHealthSchema = z.object({
  app_name: z.string(),
  app_version: z.string(),
  engine_sources: z.array(z.string()),
})
export type AppHealthDto = z.infer<typeof AppHealthSchema>

export const SourceAvailabilitySchema = z.object({
  source: z.string(),
  available: z.boolean(),
})
export type SourceAvailabilityDto = z.infer<typeof SourceAvailabilitySchema>

// ── policy ───────────────────────────────────────────────────────────────

export const CandidateEvidenceDtoSchema = z.object({
  isOfficialRepository: z.boolean(),
  isCommunityMaintained: z.boolean(),
  publisherVerified: z.boolean(),
  publisherSupported: z.boolean(),
  signaturePresent: z.boolean(),
  checksumPresent: z.boolean(),
  checksumValidated: z.boolean(),
  sandboxed: z.boolean(),
  permissionLevel: z.string(),
  filesystemAccess: z.string(),
  dbusAccess: z.string(),
  networkAccess: z.boolean(),
  deviceAccess: z.boolean(),
  findings: z.array(z.string()),
  installScriptPresent: z.boolean(),
  buildLogicChanged: z.boolean(),
})
export type CandidateEvidenceDto = z.infer<typeof CandidateEvidenceDtoSchema>

export const PolicyCandidateDtoSchema = z.object({
  source: z.string(),
  packageName: z.string(),
  version: z.string(),
  evidence: CandidateEvidenceDtoSchema,
})
export type PolicyCandidateDto = z.infer<typeof PolicyCandidateDtoSchema>

export const ReasonDtoSchema = z.object({
  kind: z.string(),
  detail: z.string(),
  contribution: z.number(),
})
export type ReasonDto = z.infer<typeof ReasonDtoSchema>

export const WarningDtoSchema = z.object({
  kind: z.string(),
  detail: z.string(),
  severity: z.string(),
  penalty: z.number(),
})
export type WarningDto = z.infer<typeof WarningDtoSchema>

export const AlternativeDtoSchema = z.object({
  candidate: PolicyCandidateDtoSchema,
  score: z.number(),
  reasons: z.array(ReasonDtoSchema),
  warnings: z.array(WarningDtoSchema),
})
export type AlternativeDto = z.infer<typeof AlternativeDtoSchema>

export const RecommendationDtoSchema = z.object({
  recommended: PolicyCandidateDtoSchema.nullable(),
  confidence: z.string(),
  reasons: z.array(ReasonDtoSchema),
  warnings: z.array(WarningDtoSchema),
  alternatives: z.array(AlternativeDtoSchema),
  score: z.number(),
})
export type RecommendationDto = z.infer<typeof RecommendationDtoSchema>

export const EvaluatePolicyResponseSchema = z.object({
  recommendation: RecommendationDtoSchema,
})
export type EvaluatePolicyResponseDto = z.infer<typeof EvaluatePolicyResponseSchema>

export const PresetDtoSchema = z.object({
  id: z.string(),
  description: z.string(),
})
export type PresetDto = z.infer<typeof PresetDtoSchema>

// ── transactions (read-only preview) ────────────────────────────────────

export const OperationDtoSchema = z.object({
  kind: z.string(),
  summary: z.string(),
  requiresPrivileges: z.boolean(),
})
export type OperationDto = z.infer<typeof OperationDtoSchema>

export const TransactionPlanDtoSchema = z.object({
  id: z.string(),
  source: z.string(),
  packageName: z.string(),
  packageVersion: z.string(),
  privilegesRequired: z.boolean(),
  expectedDownloadSize: z.number().nullable(),
  expectedDiskChange: z.number().nullable(),
  operations: z.array(OperationDtoSchema),
  state: z.string(),
  createdAt: z.string(),
  summary: z.string(),
})
export type TransactionPlanDto = z.infer<typeof TransactionPlanDtoSchema>

export const PreviewTransactionResponseSchema = z.object({
  plan: TransactionPlanDtoSchema,
  preview: z.string(),
})
export type PreviewTransactionResponseDto = z.infer<typeof PreviewTransactionResponseSchema>

export const ValidationDtoSchema = z.object({
  valid: z.boolean(),
  message: z.string(),
  privilegesRequired: z.boolean(),
})
export type ValidationDto = z.infer<typeof ValidationDtoSchema>
