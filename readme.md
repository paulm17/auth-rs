# Full-Stack Authentication System

A comprehensive authentication system built with Rust and modern web technologies, providing multiple authentication methods and security features.

## Features

### Token generation
- [x] Use rusty_paseto instead of jsonwebtoken, as paseto is considered more secure 

### Basic Email/Password Authentication
- [x] Login with Email/Password
- [x] User Registration and Login
  - [x] Email Confirmation Step
- [x] Logout functionality
  - [x] Token Blacklisting
  - [x] Token Removal
- [x] Password Recovery
  - [x] Forget Password Flow
  - [x] Reset Password Implementation
- [x] Access Token Rotation
  - [x] Automatic rotation on refresh
  - [x] Previous token blacklisting
- [x] Automatic logout on refresh token expiration

### Client Integration
- [x] JavaScript client implementation (better-auth pattern)
- [x] TanStack Router Guard integration

### Email Templates
- [x] Account Confirmation
- [x] Magic Link Authentication
- [x] Password Reset
- [ ] User Invitation (Skipped)

### Social Authentication
- [x] Amazon
- [x] Facebook/Instagram
- [x] Twitter/X
- [x] Google
- [x] LinkedIn
- [x] Reddit
- [x] GitHub
- [ ] TikTok (Requires Domain)
- [x] Twitch (Rust Implementation Issue)
- [ ] Microsoft Azure (Requires Azure account)

### Magic Link Authentication
- [x] Basic Implementation
- [ ] Ios Support
  - [ ] Initiation (Chrome)
  - [ ] Redirection (Safari)
  - [ ] Verification Code for iOS

### Future Implementations
- [ ] OTP/2FA Support
  - [ ] Base Implementation
  - [ ] WhatsApp Integration
- [x] System Tracing
- [ ] Comprehensive Testing Suite
- [x] Generate access and refresh tokens on startup

## Dependencies

- Rust/Axum (Backend)
- OAuth2-rs for social authentication
- Rusty_Paseto for token generation
- Diesel Orm for database interaction
- Frontend framework (Specify your choice)

## Documentation Links

- [OAuth2-rs Documentation](https://github.com/ramosbugs/oauth2-rs)

## Setup and Installation

(Add your setup instructions here)

## License

MIT