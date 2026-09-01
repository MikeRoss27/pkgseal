import { queryOptions, useQuery } from "@tanstack/react-query"
import { previewTransaction, type PreviewTransactionResponseDto } from "@/services/ipc/client"

export function transactionPreviewQueryOptions(input: {
  source: string
  packageName: string
  version: string
  appId?: string
  reason?: string
}) {
  return queryOptions({
    queryKey: ["transaction", "preview", input],
    queryFn: () => previewTransaction(input),
    enabled: Boolean(input.source && input.packageName && input.version),
    staleTime: 1000 * 60 * 5,
  })
}

export function useTransactionPreview(input: {
  source: string
  packageName: string
  version: string
  appId?: string
  reason?: string
}) {
  return useQuery(transactionPreviewQueryOptions(input))
}

export type { PreviewTransactionResponseDto }
