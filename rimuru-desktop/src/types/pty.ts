export type PtySessionStatus = "Starting" | "Running" | "Completed" | "Failed" | "Terminated";

export interface PtySessionInfo {
  id: string;
  agent_type: string;
  agent_name: string;
  working_dir: string;
  started_at: string;
  status: PtySessionStatus;
  pid: number | null;
  cumulative_cost_usd: number;
  token_count: number;
}

export interface LaunchRequest {
  agent_type: string;
  executable?: string;
  args?: string[];
  working_dir: string;
  cols?: number;
  rows?: number;
  initial_prompt?: string;
}

export interface PtyOutputPayload {
  session_id: string;
  data: string;
}

export interface PtyExitPayload {
  session_id: string;
  exit_code: number | null;
  success: boolean;
}
