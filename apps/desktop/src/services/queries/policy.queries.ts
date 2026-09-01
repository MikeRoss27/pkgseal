import { queryOptions, useQuery } from "@tanstack/react-query"
import {
  evaluatePolicy,
  listPolicyPresets,
  type EvaluatePolicyResponseDto,
  type PolicyCandidateDto,
} from "@/services/ipc/client"

export function policyPresetsQueryOptions() {
  return queryOptions({
    queryKey: ["policy", "presets"],
    queryFn: listPolicyPresets,
    staleTime: 1000 * 60 * 30,
  })
}

export function usePolicyPresets() {
  return useQuery(policyPresetsQueryOptions())
}

export function policyEvaluationQueryOptions(preset: string, candidates: PolicyCandidateDto[]) {
  return queryOptions({
    queryKey: ["policy", "evaluate", preset, candidates],
    queryFn: () => evaluatePolicy(preset, candidates),
    enabled: candidates.length > 0,
    staleTime: 1000 * 60 * 5,
  })
}

export function usePolicyEvaluation(preset: string, candidates: PolicyCandidateDto[]) {
  return useQuery(policyEvaluationQueryOptions(preset, candidates))
}

export type { EvaluatePolicyResponseDto }
