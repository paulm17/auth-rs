export type LinkedInScopes =
// Basic OpenID Connect
| "openid"
| "profile"  // Basic profile info (name, photo)
| "email"

// Social Actions (API v2)
| "w_member_social"   // Post content, comment, like
| "r_member_social"   // Read user's social actions
| "r_organization_social"  // Read company pages

// Company Management
| "w_organization_social"  // Post on company pages
| "rw_organization_admin"  // Full company page management

// Advertising
| "r_ads"               // Read ad accounts
| "r_ads_reporting"     // Access ad reports
| "w_ads"               // Manage ads

// Job Postings
| "r_jobs"              // Read job postings
| "w_job_postings"      // Create/manage jobs

// School Pages
| "r_school_admin"      // Manage educational institutions

// Enterprise Solutions
| "r_1st_connections_size"  // See network size
| "r_liteprofile"       // Basic profile (replaces r_basicprofile)
| "r_additional_info";  // Birthdate & phone number