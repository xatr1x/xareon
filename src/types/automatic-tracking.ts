export interface PlatformCapabilities {
  automaticProcessTracking: boolean;
}

export interface ExecutableBinding {
  id: number;
  gameId: number;
  executablePath: string;
  executableName: string;
  createdAt: string;
}

export interface RunningProcess {
  pid: number;
  executablePath: string;
  executableName: string;
  windowTitle: string | null;
  hasVisibleWindow: boolean;
}

export type AutomaticTrackingState =
  | "disabled"
  | "process_not_running"
  | "waiting_for_activity"
  | "tracking"
  | "afk"
  | "suppressed";

export interface AutomaticTrackingStatus {
  available: boolean;
  enabled: boolean;
  state: AutomaticTrackingState;
}
