import { invoke } from "@tauri-apps/api/core";

export type PreferenceEntry = {
  name: string;
  value: string;
};

export const EXPORT_DESTINATION_PREFERENCE = "export_destination_dir";

export async function getPreference(
  name: string,
): Promise<PreferenceEntry | null> {
  return invoke<PreferenceEntry | null>("get_preference", {
    getReq: { name },
  });
}

export async function setPreference(
  name: string,
  value: string,
): Promise<PreferenceEntry> {
  return invoke<PreferenceEntry>("set_preference", {
    setReq: { name, value },
  });
}
