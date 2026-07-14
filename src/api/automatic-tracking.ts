import { invoke } from "@tauri-apps/api/core";
import type {
  AutomaticTrackingStatus,
  ExecutableBinding,
  PlatformCapabilities,
  RunningProcess,
} from "../types/automatic-tracking";

export const automaticTrackingApi = {
  capabilities: (): Promise<PlatformCapabilities> => invoke("get_platform_capabilities"),
  runningProcesses: (): Promise<RunningProcess[]> => invoke("list_running_game_processes"),
  bindings: (gameId: number): Promise<ExecutableBinding[]> =>
    invoke("list_game_executable_bindings", { gameId }),
  addBinding: (gameId: number, executablePath: string): Promise<ExecutableBinding> =>
    invoke("add_game_executable_binding", { gameId, executablePath }),
  deleteBinding: (gameId: number, bindingId: number): Promise<void> =>
    invoke("delete_game_executable_binding", { gameId, bindingId }),
  setEnabled: (gameId: number, enabled: boolean): Promise<void> =>
    invoke("set_automatic_tracking_enabled", { gameId, enabled }),
  status: (gameId: number): Promise<AutomaticTrackingStatus> =>
    invoke("get_automatic_tracking_status", { gameId }),
};
