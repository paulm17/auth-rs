export type TikTokScopes =
  // User Data
  | 'user.info.basic'
  | 'user.info.profile'
  | 'user.info.stats'
  | 'user.settings'
  | 'user.account.info'
  
  // Videos
  | 'video.list'
  | 'video.upload'
  | 'video.publish'
  | 'video.delete'
  | 'video.info'
  
  // Comments & Interactions
  | 'comment.list'
  | 'comment.create'
  | 'comment.delete'
  | 'video.like'
  
  // Analytics
  | 'research.api'
  | 'user.insights'
  | 'video.insights'
  
  // Business
  | 'business.profile'
  | 'business.data'
  
  // Live Streaming
  | 'live.stream'
  | 'live.info';