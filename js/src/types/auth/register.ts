export interface RegisterCallbacks {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export interface RegisterCredentials {
  name: string;
  email: string;
  password: string;
}