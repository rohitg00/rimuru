import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  commands,
  Skill,
  InstalledSkill,
  SkillSearchResult,
  SkillRecommendation,
  SkillSearchFilters,
  SkillInstallRequest,
  SkillTranslateRequest,
  TranslationResult,
} from "@/lib/tauri";

export function useInstalledSkills(agent?: string) {
  return useQuery<InstalledSkill[]>({
    queryKey: ["skills", "installed", agent],
    queryFn: () => commands.getInstalledSkills(agent),
  });
}

export function useSearchSkills(filters?: SkillSearchFilters, enabled = true) {
  return useQuery<SkillSearchResult>({
    queryKey: ["skills", "search", filters],
    queryFn: () => commands.searchSkills(filters),
    enabled,
  });
}

export function useSkillDetails(skillId: string) {
  return useQuery<Skill | null>({
    queryKey: ["skills", "details", skillId],
    queryFn: () => commands.getSkillDetails(skillId),
    enabled: !!skillId,
  });
}

export function useSkillRecommendations(workflow?: string) {
  return useQuery<SkillRecommendation[]>({
    queryKey: ["skills", "recommendations", workflow],
    queryFn: () => commands.getSkillRecommendations(workflow),
  });
}

export function useInstallSkill() {
  const queryClient = useQueryClient();

  return useMutation<InstalledSkill, Error, SkillInstallRequest>({
    mutationFn: commands.installSkill,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "installed"] });
      queryClient.invalidateQueries({ queryKey: ["skills", "search"] });
    },
  });
}

export function useUninstallSkill() {
  const queryClient = useQueryClient();

  return useMutation<boolean, Error, { skillId: string; agent?: string }>({
    mutationFn: ({ skillId, agent }) => commands.uninstallSkill(skillId, agent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "installed"] });
      queryClient.invalidateQueries({ queryKey: ["skills", "search"] });
    },
  });
}

export function useTranslateSkill() {
  const queryClient = useQueryClient();

  return useMutation<TranslationResult, Error, SkillTranslateRequest>({
    mutationFn: commands.translateSkill,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "installed"] });
    },
  });
}

export function useEnableSkill() {
  const queryClient = useQueryClient();

  return useMutation<boolean, Error, { skillId: string; agent?: string }>({
    mutationFn: ({ skillId, agent }) => commands.enableSkill(skillId, agent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "installed"] });
    },
  });
}

export function useDisableSkill() {
  const queryClient = useQueryClient();

  return useMutation<boolean, Error, { skillId: string; agent?: string }>({
    mutationFn: ({ skillId, agent }) => commands.disableSkill(skillId, agent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills", "installed"] });
    },
  });
}
