import type { IssuerConfig } from "../issuers";

export function formatEnvironment(env?: IssuerConfig["environment"]): string {
  switch (env) {
    case "production":
      return "Production";
    case "staging":
    default:
      return "Sandbox";
  }
}

export function formatIssuerType(type?: IssuerConfig["issuer_type"]): string {
  switch (type) {
    case "acme":
    default:
      return "ACME";
  }
}

