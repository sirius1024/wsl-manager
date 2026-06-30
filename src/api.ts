import { invoke } from "@tauri-apps/api/core";
import type { WslInstance, WslVersion } from "./types";

export const logMessage = (msg: string) => invoke<void>("log_message", { msg });

export const listInstances = () => invoke<WslInstance[]>("list_instances");

export const getWslVersion = () => invoke<WslVersion>("get_wsl_version");

export const listOnlineDistributions = () =>
  invoke<string[]>("list_online_distributions");

export const installDistribution = (distro: string, installName: string) =>
  invoke<void>("install_distribution", { distro, installName });

export const renameInstance = (oldName: string, newName: string) =>
  invoke<void>("rename_instance", { oldName, newName });

export const startInstance = (name: string) =>
  invoke<void>("start_instance", { name });

export const stopInstance = (name: string) =>
  invoke<void>("stop_instance", { name });

export const shutdown = () => invoke<void>("shutdown");

export const openTerminal = (name: string) =>
  invoke<void>("open_terminal", { name });

export const setDefaultInstance = (name: string) =>
  invoke<void>("set_default_instance", { name });

export const deleteInstance = (name: string) =>
  invoke<void>("delete_instance", { name });
