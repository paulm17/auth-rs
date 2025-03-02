export interface CodeResponse {
  is_valid: boolean;
}

export interface CodeCallbacks {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}