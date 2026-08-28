import { queryOptions, useQuery } from "@tanstack/react-query"
import { resolveApplications, type ResolveResponseDto } from "@/services/ipc/client"

/** Queries shorter than this are too broad to hit external sources with. */
export const MIN_SEARCH_QUERY_LENGTH = 2

export function resolvedApplicationsQueryOptions(query: string) {
  const trimmed = query.trim()
  return queryOptions({
    queryKey: ["resolver", "applications", trimmed],
    queryFn: () => resolveApplications(trimmed),
    enabled: trimmed.length >= MIN_SEARCH_QUERY_LENGTH,
    staleTime: 1000 * 60 * 5,
    retry: false,
  })
}

export function useResolvedApplications(query: string) {
  return useQuery(resolvedApplicationsQueryOptions(query))
}

export type { ResolveResponseDto }
