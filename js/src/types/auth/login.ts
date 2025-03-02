export interface LoginCallbacks {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface MagicLinkCallbacks {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export interface MagicLinkCredentials {
  email: string;
  redirectTo: string;
}