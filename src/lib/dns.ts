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
