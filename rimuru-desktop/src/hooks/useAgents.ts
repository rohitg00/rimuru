import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands, AgentWithStatus, Agent, AddAgentRequest } from "@/lib/tauri";

export function useAgents() {
  return useQuery<AgentWithStatus[]>({
    queryKey: ["agents"],
    queryFn: commands.getAgents,
  });
}

export function useAgent(agentId: string) {
  return useQuery<AgentWithStatus | null>({
    queryKey: ["agents", agentId],
    queryFn: () => commands.getAgentDetails(agentId),
    enabled: !!agentId,
  });
}

export function useScanAgents() {
  const queryClient = useQueryClient();

  return useMutation<Agent[], Error>({
    mutationFn: commands.scanAgents,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["agents"] });
    },
  });
}

export function useAddAgent() {
  const queryClient = useQueryClient();

  return useMutation<AgentWithStatus, Error, AddAgentRequest>({
    mutationFn: commands.addAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["agents"] });
    },
  });
}
