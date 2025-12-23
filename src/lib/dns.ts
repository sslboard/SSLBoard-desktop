import { invoke } from "@tauri-apps/api/core";

export type PrepareDnsChallengeRequest = {
  domain: string;
  txt_value: string;
};

export type PreparedDnsChallenge = {
  record: {
    adapter: string;
    record_name: string;
    value: string;
    zone: string;
  };
};

export type PropagationState =
  | "pending"
  | "found"
  | "nx_domain"
  | "wrong_content"
  | "error";

export type PropagationResult = {
  state: PropagationState;
  reason?: string;
  observed_values: string[];
};

export async function prepareDnsChallenge(
  req: PrepareDnsChallengeRequest,
): Promise<PreparedDnsChallenge> {
  return invoke<PreparedDnsChallenge>("prepare_dns_challenge", {
    prepareReq: req,
  });
}

export async function checkDnsPropagation(
  domain: string,
  txtValue: string,
): Promise<PropagationResult> {
  return invoke<PropagationResult>("check_dns_propagation", {
    checkReq: { domain, txt_value: txtValue },
  });
}
