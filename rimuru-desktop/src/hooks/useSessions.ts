import { useQuery } from "@tanstack/react-query";
import { commands, Session, SessionFilters } from "@/lib/tauri";

export function useSessions(filters?: SessionFilters) {
  return useQuery<Session[]>({
    queryKey: ["sessions", filters],
    queryFn: () => commands.getSessions(filters),
  });
}

export function useSession(sessionId: string) {
  return useQuery<Session | null>({
    queryKey: ["sessions", sessionId],
    queryFn: () => commands.getSessionDetails(sessionId),
    enabled: !!sessionId,
  });
}

export function useActiveSessions() {
  return useQuery<Session[]>({
    queryKey: ["sessions", "active"],
    queryFn: commands.getActiveSessions,
    refetchInterval: 10000,
  });
}
