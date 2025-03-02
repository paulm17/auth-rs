import { User } from "./user";

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
}

export interface SessionResponse {
  user: User;
}

export interface ExtendedRequestInit extends RequestInit {
  _retry?: boolean;
}

export * from "./login";
export * from "./register";
export * from "./user";
export * from "./getCode";