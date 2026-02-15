import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands, InstallPluginRequest, ConfigurePluginRequest } from "@/lib/tauri";

export function usePlugins(showAvailable?: boolean, capability?: string) {
  return useQuery({
    queryKey: ["plugins", showAvailable, capability],
    queryFn: () => commands.getPlugins(showAvailable, capability),
  });
}

export function usePluginDetails(pluginId: string) {
  return useQuery({
    queryKey: ["plugin", pluginId],
    queryFn: () => commands.getPluginDetails(pluginId),
    enabled: !!pluginId,
  });
}

export function usePluginConfig(pluginId: string) {
  return useQuery({
    queryKey: ["pluginConfig", pluginId],
    queryFn: () => commands.getPluginConfig(pluginId),
    enabled: !!pluginId,
  });
}

export function usePluginEvents(pluginId?: string, limit?: number) {
  return useQuery({
    queryKey: ["pluginEvents", pluginId, limit],
    queryFn: () => commands.getPluginEvents(pluginId, limit),
  });
}

export function useInstallPlugin() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: InstallPluginRequest) => commands.installPlugin(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins"] });
    },
  });
}

export function useEnablePlugin() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (pluginId: string) => commands.enablePlugin(pluginId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins"] });
    },
  });
}

export function useDisablePlugin() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (pluginId: string) => commands.disablePlugin(pluginId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins"] });
    },
  });
}

export function useUninstallPlugin() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ pluginId, force }: { pluginId: string; force?: boolean }) =>
      commands.uninstallPlugin(pluginId, force),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins"] });
    },
  });
}

export function useConfigurePlugin() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: ConfigurePluginRequest) => commands.configurePlugin(request),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["pluginConfig", variables.plugin_id] });
      queryClient.invalidateQueries({ queryKey: ["plugins"] });
    },
  });
}
