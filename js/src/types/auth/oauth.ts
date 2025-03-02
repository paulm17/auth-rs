import { AmazonScopes } from './social/amazon';
import { FacebookScopes } from './social/facebook';
import { GitHubScopes } from './social/github';
import { GoogleScopes } from './social/google';
import { InstagramScopes } from './social/instagram';
import { LinkedInScopes } from './social/linkedin';
import { MicrosoftScopes } from './social/microsoft';
import { RedditScopes } from './social/reddit';
import { TikTokScopes } from './social/tiktok';
import { TwitchScopes } from './social/twitch';
import { TwitterScopes } from './social/twitter';

export type OAuthProvider =
  | 'amazon'
  | 'github'
  | 'google'
  | 'facebook'
  | 'instagram'
  | 'linkedin'
  | 'microsoft'
  | 'reddit'
  | 'tiktok'
  | 'twitter'
  | 'twitch';

// Base interface without scopes
interface BaseOAuthOptions {
  callback_url: string;
}

// Amazon specific options
interface AmazonOAuthOptions extends BaseOAuthOptions {
  provider: 'amazon';
  callback_url: string;
  scopes?: AmazonScopes[];
}

// Facebook specific options
interface FacebookOAuthOptions extends BaseOAuthOptions {
  provider: 'facebook';
  callback_url: string;
  scopes?: FacebookScopes[];
}

// GitHub specific options
interface GitHubOAuthOptions extends BaseOAuthOptions {
  provider: 'github';
  callback_url: string;
  scopes?: GitHubScopes[];
}

// Google specific options
interface GoogleOAuthOptions extends BaseOAuthOptions {
  provider: 'google';
  callback_url: string;
  scopes?: GoogleScopes[];
}

// Linkedin specific options
interface LinkedInOAuthOptions extends BaseOAuthOptions {
  provider: 'linkedin';
  callback_url: string;
  scopes?: LinkedInScopes[];
}

// Instagram specific options
interface InstagramOAuthOptions extends BaseOAuthOptions {
  provider: 'instagram';
  callback_url: string;
  scopes?: InstagramScopes[];
}

// Microsoft specific options
interface MicrosoftOAuthOptions extends BaseOAuthOptions {
  provider: 'microsoft';
  callback_url: string;
  scopes?: MicrosoftScopes[];
}

// Reddit specific options
interface RedditOAuthOptions extends BaseOAuthOptions {
  provider: 'reddit';
  callback_url: string;
  scopes?: RedditScopes[];
}

// Tiktok specific options
interface TiktokOAuthOptions extends BaseOAuthOptions {
  provider: 'tiktok';
  callback_url: string;
  scopes?: TikTokScopes[];
}

// Twitch specific options
interface TwitchOAuthOptions extends BaseOAuthOptions {
  provider: 'twitch';
  callback_url: string;
  scopes?: TwitchScopes[];
}

// Twitter specific options
interface TwitterOAuthOptions extends BaseOAuthOptions {
  provider: 'twitter';
  callback_url: string;
  scopes?: TwitterScopes[];
}

export type OAuthOptions = 
| AmazonOAuthOptions 
| GitHubOAuthOptions 
| GoogleOAuthOptions
| FacebookOAuthOptions 
| LinkedInOAuthOptions 
| InstagramOAuthOptions
| MicrosoftOAuthOptions
| RedditOAuthOptions 
| TiktokOAuthOptions
| TwitchOAuthOptions 
| TwitterOAuthOptions;

export interface OAuthCallbacks {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export interface OAuthResponse {
  url: string;
}