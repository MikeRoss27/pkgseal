import { invoke } from "@tauri-apps/api/core"
import { z } from "zod"
import {
  AppHealthSchema,
  EvaluatePolicyResponseSchema,
  PreviewTransactionResponseSchema,
  PresetDtoSchema,
  ResolveResponseSchema,
  SourceAvailabilitySchema,
  ValidationDtoSchema,
  type AppHealthDto,
  type CandidateEvidenceDto,
  type CandidateRefDto,
  type EvaluatePolicyResponseDto,
  type MatchConfidenceLevel,
  type MatchSignalType,
  type PackageDetailsDto,
  type PackageSummaryDto,
  type PolicyCandidateDto,
  type PresetDto,
  type PreviewTransactionResponseDto,
  type RecommendationDto,
  type ResolveResponseDto,
  type ResolvedApplicationDto,
  type SignalDto,
  type SourceAvailabilityDto,
  type TransactionPlanDto,
  type ValidationDto,
} from "./schemas"

function isTauriRuntime(): boolean {
  if (typeof window !== "undefined" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window)) return true
  // Vitest mocks `invoke` – allow the mocked path to run in tests even without a real WebView
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  if ((import.meta as any).env?.MODE === "test") return true
  // Fallback for other test runners
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  if (typeof process !== "undefined" && (process as any).env?.VITEST) return true
  return false
}

async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!isTauriRuntime()) return null
  // Avoid passing `undefined` as second arg – Tauri's mock tests expect single-arg calls for no-arg commands
  if (args === undefined) {
    return await invoke<T>(cmd)
  }
  return await invoke<T>(cmd, args)
}

// Re-export DTO types so existing imports from "./client" keep working.
export type {
  AppHealthDto,
  CandidateEvidenceDto,
  CandidateRefDto,
  EvaluatePolicyResponseDto,
  MatchConfidenceLevel,
  MatchSignalType,
  PackageDetailsDto,
  PackageSummaryDto,
  PolicyCandidateDto,
  PresetDto,
  PreviewTransactionResponseDto,
  RecommendationDto,
  ResolveResponseDto,
  ResolvedApplicationDto,
  SignalDto,
  SourceAvailabilityDto,
  TransactionPlanDto,
  ValidationDto,
}

export async function getAppHealth(): Promise<AppHealthDto> {
  const raw = await safeInvoke<unknown>("app_health")
  if (raw === null) {
    // Browser-only fallback – keep UI usable without Tauri
    return { app_name: "PkgSeal", app_version: "0.1.0-alpha", engine_sources: ["arch-official", "aur", "flatpak"] }
  }
  return AppHealthSchema.parse(raw)
}

export async function getSourceAvailability(): Promise<SourceAvailabilityDto[]> {
  const raw = await safeInvoke<unknown>("source_availability")
  if (raw === null) {
    // In browser dev, return empty so StatusBar shows placeholder instead of error
    return []
  }
  return zArray(SourceAvailabilitySchema).parse(raw)
}

export async function resolveApplications(query: string): Promise<ResolveResponseDto> {
  const raw = await safeInvoke<unknown>("resolve_applications_command", {
    request: { query },
  })
  if (raw === null) {
    // Browser fallback – no Tauri runtime. Throw a user-friendly error that the UI
    // can render via InlineError instead of a raw IPC fetch failure.
    throw new Error("Tauri runtime unavailable — run with `cargo tauri dev` to enable native search")
  }
  return ResolveResponseSchema.parse(raw)
}

export async function evaluatePolicy(
  preset: string,
  candidates: PolicyCandidateDto[],
): Promise<EvaluatePolicyResponseDto> {
  const raw = await safeInvoke<unknown>("evaluate_policy", {
    request: { preset, candidates },
  })
  if (raw === null) throw new Error("Tauri runtime unavailable")
  return EvaluatePolicyResponseSchema.parse(raw)
}

export async function listPolicyPresets(): Promise<PresetDto[]> {
  const raw = await safeInvoke<unknown>("list_policy_presets")
  if (raw === null) {
    return [
      { id: "balanced", description: "Balances provenance, publisher support, sandboxing, permissions" },
      { id: "native-first", description: "Prefers native packages when trust comparable" },
      { id: "sandbox-first", description: "Prefers sandboxed when permissions reasonable" },
      { id: "maximum-review", description: "Requires stronger review for community/broad" },
    ]
  }
  return zArray(PresetDtoSchema).parse(raw)
}

export async function previewTransaction(input: {
  source: string
  packageName: string
  version: string
  appId?: string
  reason?: string
}): Promise<PreviewTransactionResponseDto> {
  const raw = await safeInvoke<unknown>("preview_transaction", {
    request: {
      source: input.source,
      packageName: input.packageName,
      version: input.version,
      appId: input.appId ?? null,
      reason: input.reason ?? null,
    },
  })
  if (raw === null) throw new Error("Tauri runtime unavailable")
  return PreviewTransactionResponseSchema.parse(raw)
}

export async function validateTransactionRequest(input: {
  source: string
  packageName: string
  version: string
  appId?: string
}): Promise<ValidationDto> {
  const raw = await safeInvoke<unknown>("validate_transaction_request", {
    request: {
      source: input.source,
      packageName: input.packageName,
      version: input.version,
      appId: input.appId ?? null,
      reason: null,
    },
  })
  if (raw === null) return { valid: true, message: "browser fallback", privilegesRequired: input.source !== "flatpak" }
  return ValidationDtoSchema.parse(raw)
}

// ── internal helpers ────────────────────────────────────────────────

function zArray<T extends z.ZodTypeAny>(schema: T) {
  return z.array(schema)
}