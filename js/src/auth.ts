import { AuthResponse, CodeCallbacks, CodeResponse, LoginCallbacks, LoginCredentials, MagicLinkCallbacks, MagicLinkCredentials, RegisterCallbacks, RegisterCredentials, SessionResponse } from "./types/auth";
import { ForgetPasswordCallbacks, ForgetPasswordCredentials, ForgetPasswordResponse } from "./types/auth/forgetPassword";
import { LogoutCallbacks } from "./types/auth/logout";
import { OAuthCallbacks, OAuthOptions, OAuthResponse } from "./types/auth/oauth";
import { ResetPasswordCallbacks, ResetPasswordCredentials, ResetPasswordResponse } from "./types/auth/resetPassword";
import { BaseClient } from "./lib";

export class Auth extends BaseClient {
  constructor(config: { baseURL: string }) {
    super(config);
  }
  public signIn = {
    email: async (
      credentials: LoginCredentials,
      callbacks?: LoginCallbacks
    ): Promise<{ error?: Error }> => {
      try {
        const response = await this.fetchWithAuth('/login', {
          method: 'POST',
          body: JSON.stringify(credentials),
        });

        if (!response.ok) {
          const error = new Error(`Login failed: ${response.statusText}`);
          callbacks?.onError?.(error);
          return { error };
        }

        const data: AuthResponse = await response.json();
        this.accessToken = data.access_token;

        const session = await this.getSession();
        if (!session.data) {
          const error = new Error('Failed to get user data after login');
          callbacks?.onError?.(error);
          return { error };
        }

        callbacks?.onSuccess?.();
        return {};
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Login failed');
        callbacks?.onError?.(err);
        return { error: err };
      }
    },

    magicLink: async (
      credentials: MagicLinkCredentials,
      callbacks?: MagicLinkCallbacks
    ): Promise<{ error?: Error }> => {
      try {
        const response = await this.fetchWithAuth('/generate_magiclink', {
          method: 'POST',
          body: JSON.stringify(credentials),
        });

        if (!response.ok) {
          const error = new Error(`Magic link creation failed: ${response.statusText}`);
          callbacks?.onError?.(error);
          return { error };
        }

        const data: AuthResponse = await response.json();
        this.accessToken = data.access_token;

        const session = await this.getSession();
        if (!session.data) {
          const error = new Error('Failed to get magic link response');
          callbacks?.onError?.(error);
          return { error };
        }

        callbacks?.onSuccess?.();
        return {};
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Login failed');
        callbacks?.onError?.(err);
        return { error: err };
      }
    },

    social: async (
      options: OAuthOptions,
      callbacks?: OAuthCallbacks
    ): Promise<{ data?: OAuthResponse; error?: Error } | null> => {
      try {
        let requiredScopes: string[] = [];
    
        // Set default scopes based on provider
        switch (options.provider) {
          case 'amazon':
            requiredScopes = ['profile', 'profile:user_id'];
            break;
          case 'facebook':
            requiredScopes = ['email', 'public_profile'];
            break;
          case 'github':
            requiredScopes = ['public_repo', 'user:email'];
            break;
          case 'google':
            requiredScopes = ['email', 'profile', 'openid'];
            break;
          case 'instagram':
            requiredScopes = ['user_profile', 'user_media'];
            break;          
          case 'linkedin':
            requiredScopes = ['profile', 'email', 'openid'];
            break;
          case 'microsoft':
            requiredScopes = ['User.Read', 'email', 'profile', 'openid'];
            break;
          case 'reddit':
            requiredScopes = ['identity', 'read'];
            break;
          case 'tiktok':
            requiredScopes = ['user.info.basic'];
            break;
          case 'twitch':
            requiredScopes = ['user:read:email'];
            break;
          case 'twitter':
            requiredScopes = ['tweet.read', 'users.read', 'offline.access'];
            break;
        }

        // Combine user-provided scopes with required scopes
        options.scopes = [
          ...new Set([
            ...(options.scopes || []),
            ...requiredScopes
          ])
        ];

        const response = await this.fetchWithAuth(`/oauth/url`, {
          method: 'POST',
          body: JSON.stringify({
            provider: options.provider,
            callback_url: options.callback_url,
            scopes: options.scopes.join(",")
          }),
        });

        if (!response.ok) {
          const error = new Error(`Login failed: ${response.statusText}`);
          callbacks?.onError?.(error);
          return { error };
        }

        const data: OAuthResponse = await response.json();
        window.location.href = data.url;
        
        return null;
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Social authentication url generation failed');
        callbacks?.onError?.(err);
        return { error: err };
      }
    }
  };

  public signUp = {
    email: async (
      credentials: RegisterCredentials,
      callbacks?: RegisterCallbacks
    ): Promise<{ error?: Error }> => {
      try {
        const response = await this.fetchWithAuth('/register', {
          method: 'POST',
          body: JSON.stringify(credentials),
        });
    
        if (!response.ok) {
          const error = new Error(`Registration failed: ${response.statusText}`);
          callbacks?.onError?.(error);
          return { error };
        }
    
        // After successful registration, attempt to login
        const loginResult = await this.signIn.email(credentials, {
          onError: (error: any) => {
            callbacks?.onError?.(new Error('Registration successful but auto-login failed: ' + error.message));
          }
        });
    
        if (loginResult.error) {
          return loginResult;
        }
    
        callbacks?.onSuccess?.();
        return {};
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Registration failed');
        callbacks?.onError?.(err);
        return { error: err };
      }
    }
  };

  public signOut = async (
    callbacks?: LogoutCallbacks
  ): Promise<{ error?: Error }> => {
    try {
      await this.fetchWithAuth('/logout', { method: 'GET' });
  
      // Clear accessToken
      this.accessToken = null;

      // (Optional) Also abort any in-flight requests if you want to ensure
      // no further requests proceed after signOut:
      this.abortAllRequests();

      callbacks?.onSuccess?.();
      return {};
    } catch (error) {
      const err = error instanceof Error ? error : new Error('logout failed');
      callbacks?.onError?.(err);
      return { error: err };
    }
  };

  public forgetPassword = async (
    credentials: ForgetPasswordCredentials,
    callbacks?: ForgetPasswordCallbacks
  ): Promise<{ data?: ForgetPasswordResponse; error?: Error }> => {
    try {
      const response = await this.fetchWithAuth('/forgot_password', {
        method: 'POST',
        body: JSON.stringify(credentials),
      });

      if (!response.ok) {
        const error = new Error(`Forget password request failed: ${response.statusText}`);
        callbacks?.onError?.(error);
        return { error };
      }

      const data: ForgetPasswordResponse = await response.json();
      callbacks?.onSuccess?.();
      return { data };
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Forget password request failed');
      callbacks?.onError?.(err);
      return { error: err };
    }
  };

  public resetPassword = async (
    credentials: ResetPasswordCredentials,
    callbacks?: ResetPasswordCallbacks
  ): Promise<{ data?: ResetPasswordResponse; error?: Error }> => {
    try {
      const response = await this.fetchWithAuth('/reset_password', {
        method: 'POST',
        body: JSON.stringify(credentials),
      });

      if (!response.ok) {
        const error = new Error(`Password reset failed: ${response.statusText}`);
        callbacks?.onError?.(error);
        return { error };
      }

      const data: ResetPasswordResponse = await response.json();
      callbacks?.onSuccess?.();
      return { data };
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Password reset failed');
      callbacks?.onError?.(err);
      return { error: err };
    }
  };

  public getSession = async (): Promise<{ data: SessionResponse | null }> => {
    try {
      const response = await this.fetchWithAuth('/users/me');
      
      if (!response.ok) {
        if (response.status === 401) {
          return { data: null };
        }
        throw new Error(`Failed to get session: ${response.statusText}`);
      }

      const data: SessionResponse = await response.json();

      return { data };
    } catch (error) {
      if (error instanceof Error && error.message.includes('401')) {
        return { data: null };
      }
      throw error;
    }
  };

  public getCode = async (
    code: string,
    callbacks?: CodeCallbacks
  ): Promise<{ data: CodeResponse | null }> => {
    try {
      const response = await this.fetchWithAuth('/check_code', {
        method: 'POST',
        body: JSON.stringify({
          code
        }),
      });

      
      if (!response.ok) {
        if (response.status === 401) {
          return { data: null };
        }
        throw new Error(`Failed to get code: ${response.statusText}`);
      }

      const data: CodeResponse = await response.json();
      callbacks?.onSuccess?.();

      return { data };
    } catch (error) {
      if (error instanceof Error && error.message.includes('401')) {
        callbacks?.onError?.(error);
        return { data: null };
      }
      throw error;
    }
  };
}