export interface ForgetPasswordCredentials {
  email: string;
  redirectTo?: string;
}

export interface ForgetPasswordCallbacks {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export interface ForgetPasswordResponse {
  message: string;
}