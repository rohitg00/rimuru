import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands, TriggerHookRequest } from "@/lib/tauri";

export function useHookTypes() {
  return useQuery({
    queryKey: ["hooks"],
    queryFn: commands.getHooks,
  });
}

export function useHookHandlers(hookType?: string) {
  return useQuery({
    queryKey: ["hookHandlers", hookType],
    queryFn: () => commands.getHookHandlers(hookType),
  });
}

export function useHookExecutions(hookType?: string, limit?: number) {
  return useQuery({
    queryKey: ["hookExecutions", hookType, limit],
    queryFn: () => commands.getHookExecutions(hookType, limit),
  });
}

export function useHookStats() {
  return useQuery({
    queryKey: ["hookStats"],
    queryFn: commands.getHookStats,
  });
}

export function useTriggerHook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: TriggerHookRequest) => commands.triggerHook(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hookExecutions"] });
      queryClient.invalidateQueries({ queryKey: ["hooks"] });
    },
  });
}

export function useEnableHookHandler() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (handlerId: string) => commands.enableHookHandler(handlerId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hookHandlers"] });
      queryClient.invalidateQueries({ queryKey: ["hooks"] });
      queryClient.invalidateQueries({ queryKey: ["hookStats"] });
    },
  });
}

export function useDisableHookHandler() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (handlerId: string) => commands.disableHookHandler(handlerId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hookHandlers"] });
      queryClient.invalidateQueries({ queryKey: ["hooks"] });
      queryClient.invalidateQueries({ queryKey: ["hookStats"] });
    },
  });
}
