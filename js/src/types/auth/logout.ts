export interface LogoutCallbacks {
    onSuccess?: () => void;
    onError?: (error: Error) => void;
  }