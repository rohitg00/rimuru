import { useQuery } from "@tanstack/react-query";
import { commands, CostSummary, CostBreakdown, CostHistoryPoint, TimeRangeRequest } from "@/lib/tauri";

export function useCostSummary(timeRange?: TimeRangeRequest) {
  return useQuery<CostSummary>({
    queryKey: ["costs", "summary", timeRange],
    queryFn: () => commands.getCostSummary(timeRange),
  });
}

export function useCostBreakdown(timeRange?: TimeRangeRequest) {
  return useQuery<CostBreakdown>({
    queryKey: ["costs", "breakdown", timeRange],
    queryFn: () => commands.getCostBreakdown(timeRange),
  });
}

export function useCostHistory(days?: number) {
  return useQuery<CostHistoryPoint[]>({
    queryKey: ["costs", "history", days],
    queryFn: () => commands.getCostHistory(days),
  });
}
