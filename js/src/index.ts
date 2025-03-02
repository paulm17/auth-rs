import { SessionResponse, User } from "./types/auth";
import { Auth } from "./auth";
import { Perms } from "./perms";

class Heimdall {
  public auth: Auth;
  public perms: Perms;
  private baseURL: string;

  constructor(config: { baseURL: string }) {
    this.baseURL = config.baseURL;
    this.auth = new Auth({ baseURL: this.baseURL });
    this.perms = new Perms({ baseURL: this.baseURL });
  }
}

export const heimdall = new Heimdall({
  baseURL: process.env.AUTH_SERVER_URL as string,
});

// Export types that might be needed by consuming code
export type { User, SessionResponse };