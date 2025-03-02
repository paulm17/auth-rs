export type InstagramScopes =
// Basic Access (Graph API)
| 'instagram_basic'
| 'instagram_graph_user_profile'
| 'instagram_graph_user_media'

// Content Management
| 'instagram_content_publish'
| 'instagram_manage_comments'
| 'instagram_manage_insights'
| 'instagram_manage_metadata'  // New
| 'instagram_manage_messages'

// Shopping & Commerce
| 'instagram_shopping_tag_products'
| 'catalog_management'  // Often needed with shopping

// Page Management
| 'pages_show_list'
| 'pages_read_engagement'
| 'pages_manage_posts'  // Required for publishing
| 'pages_manage_metadata'

// Business Management
| 'business_management'
| 'instagram_manage_events'  // New
| 'read_insights'

// Advanced
| 'ads_management'  // For advertising features
| 'leads_retrieval';  // For lead ads