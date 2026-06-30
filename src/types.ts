export type InstanceState = "Running" | "Stopped" | "Unknown";

export interface WslInstance {
  name: string;
  state: InstanceState;
  version: number;
  default: boolean;
  distribution?: string | null;
  ipAddress?: string | null;
}

export interface WslVersion {
  wslVersion?: string | null;
  kernelVersion?: string | null;
  windowsVersion?: string | null;
  fields: Record<string, string>;
  raw: string;
}
