export interface ResetPasswordCredentials {
  password: string;
  code: string;
}

export interface ResetPasswordCallbacks {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export interface ResetPasswordResponse {
  message: string;
}