import { AuthResponse, ExtendedRequestInit } from "./types/auth";

export class BaseClient {
  protected accessToken: string | null = null;
  protected refreshToken: string | null = null;
  protected refreshPromise: Promise<void> | null = null;
  protected baseURL: string;

  // Global AbortController: all fetches share this signal.
  protected globalAbortController: AbortController;
  protected globalSignal: AbortSignal;

  constructor(config: { baseURL: string }) {
    this.baseURL = config.baseURL;
    this.globalAbortController = new AbortController();
    this.globalSignal = this.globalAbortController.signal;
  }

  protected getAuthHeader = (): HeadersInit => {
    return this.accessToken
      ? { 'Authorization': `Bearer ${this.accessToken}`, 'Content-Type': 'application/json' }
      : { 'Content-Type': 'application/json' };
  };

  /**
   * Abort all in-flight requests by aborting the global AbortController.
   * Then reinitialize the controller so future requests can proceed.
   */
  protected abortAllRequests = () => {
    this.globalAbortController.abort();
    this.globalAbortController = new AbortController();
    this.globalSignal = this.globalAbortController.signal;
  };

  protected async fetchWithAuth(
    endpoint: string,
    options: ExtendedRequestInit = {}
  ): Promise<Response> {
    if (this.globalSignal.aborted) {
      throw new DOMException('Global request signal is already aborted', 'AbortError');
    }

    const headers = ["/login", "/register", "/forgot_password", "/reset_password", "/check_code"].includes(endpoint)
      ? { 'Content-Type': 'application/json' }
      : this.getAuthHeader();

    const url = `${this.baseURL}${endpoint}`;
    let response: Response;
    try {
      response = await fetch(url, {
        ...options,
        headers: { ...headers, ...options.headers },
        credentials: 'include',
        signal: this.globalSignal, // all fetches use the same global signal
      });
    } catch (error) {
      if (this.globalSignal.aborted) {
        throw new DOMException('Aborted all requests', 'AbortError');
      }
      throw error;
    }

    // When the token is invalid (401) and we haven't retried yet:
    if (response.status === 401 && !options._retry) {
      // Attempt to refresh the token once.
      if (!this.refreshPromise) {
        this.refreshPromise = this.reGetAccessToken();
      }
      await this.refreshPromise;
      this.refreshPromise = null;
      
      // If the token refresh failed, accessToken will be null.
      if (!this.accessToken) {
        // Instead of throwing an error, we gracefully return the original 401 response.
        // Optionally, trigger a redirect to the login page or update global state here.
        return response;
      }
      // Otherwise, retry the original request with a flag to avoid infinite loops.
      return this.fetchWithAuth(endpoint, { ...options, _retry: true });
    }

    return response;
  }

  protected async reGetAccessToken(): Promise<void> {
    try {
      const response = await fetch(`${this.baseURL}/refresh`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        signal: this.globalSignal, // use global signal here as well
      });
      if (!response.ok) {
        // Token refresh failed: clear token and abort all in-flight requests.
        this.accessToken = null;
        this.abortAllRequests();
        // Instead of throwing an error, we simply return so that fetchWithAuth
        // can detect that accessToken is null and handle the 401 gracefully.
        return;
      }
      const data: AuthResponse = await response.json();
      this.accessToken = data.access_token;
    } catch (error) {
      // In case of network errors or if the request was aborted, treat it as a failure.
      this.accessToken = null;
      this.abortAllRequests();
      return;
    }
  }
}
