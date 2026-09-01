import { queryOptions } from "@tanstack/react-query"
import { getAppHealth, getSourceAvailability } from "@/services/ipc/client"

export const appHealthQueryOptions = queryOptions({
  queryKey: ["system", "health"],
  queryFn: getAppHealth,
  staleTime: 1000 * 60 * 30,
  retry: false,
})

export const sourceAvailabilityQueryOptions = queryOptions({
  queryKey: ["system", "source-availability"],
  queryFn: getSourceAvailability,
  staleTime: 1000 * 60 * 5,
  retry: false,
  // Don't spam Tauri invoke when running in plain browser (vite dev without Tauri)
  // – the client already returns [] in that case, but keep retry off for any
  // real IPC CSP failures that need a Tauri restart.
  throwOnError: false,
})