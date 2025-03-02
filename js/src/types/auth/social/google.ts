export type GoogleScopes =
  | "openid" // Required for OpenID Connect authentication
  | "profile" // Access to basic profile information (name, picture, etc.)
  | "email" // Access to the user's email address
  | "https://www.googleapis.com/auth/userinfo.email" // Full URL for email scope
  | "https://www.googleapis.com/auth/userinfo.profile" // Full URL for profile scope
  | "https://www.googleapis.com/auth/calendar" // Access to Google Calendar
  | "https://www.googleapis.com/auth/contacts" // Access to Google Contacts
  | "https://www.googleapis.com/auth/drive" // Access to Google Drive
  | "https://www.googleapis.com/auth/drive.file" // Access to files created by the app
  | "https://www.googleapis.com/auth/spreadsheets" // Access to Google Sheets
  | "https://www.googleapis.com/auth/youtube" // Access to YouTube
  | "https://www.googleapis.com/auth/gmail.readonly" // Read-only access to Gmail
  | "https://www.googleapis.com/auth/cloud-platform" // Access to Google Cloud Platform
  | "https://www.googleapis.com/auth/analytics.readonly" // Read-only access to Google Analytics
  | string; // Fallback for custom or future scopes