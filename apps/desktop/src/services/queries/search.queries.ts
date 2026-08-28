import { queryOptions, useQuery } from "@tanstack/react-query"
import { resolveApplications } from "@/services/ipc/client"
import type { ResolveResponseDto } from "@/services/ipc/client"

export const MIN_SEARCH_QUERY_LENGTH = 2

/** Generic search — currently backed by resolveApplications. */
export function searchQueryOptions(query: string) {
  const trimmed = query.trim()
  return queryOptions({
    queryKey: ["search", trimmed] as const,
    queryFn: (): Promise<ResolveResponseDto> => resolveApplications(trimmed),
    enabled: trimmed.length >= MIN_SEARCH_QUERY_LENGTH,
    staleTime: 1000 * 60 * 5,
  })
}

export function useSearch(query: string) {
  return useQuery(searchQueryOptions(query))
}

/** Fetch details for a single resolved application by id (via search then select). */
export function packageDetailsQueryOptions(applicationId: string, searchQuery: string) {
  const trimmedAppId = applicationId.trim()
  const trimmedQuery = searchQuery.trim()
  return queryOptions({
    queryKey: ["package", "details", trimmedAppId, trimmedQuery] as const,
    queryFn: async () => {
      const res = await resolveApplications(trimmedQuery)
      const app = res.applications.find((a) => a.id === trimmedAppId)
      if (!app) throw new Error(`Application not found: ${trimmedAppId}`)
      return app
    },
    enabled: trimmedAppId.length > 0 && trimmedQuery.length >= MIN_SEARCH_QUERY_LENGTH,
    staleTime: 1000 * 60 * 5,
  })
}

export function usePackageDetails(applicationId: string, searchQuery: string) {
  return useQuery(packageDetailsQueryOptions(applicationId, searchQuery))
}
