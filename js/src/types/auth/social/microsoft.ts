export type MicrosoftScopes =
  // User Profile
  | 'User.Read'
  | 'User.ReadWrite'
  | 'User.ReadBasic.All'
  | 'profile'
  | 'openid'
  | 'email'
  | 'offline_access'

  // Calendar
  | 'Calendars.Read'
  | 'Calendars.ReadWrite'
  | 'Calendars.Read.Shared'
  | 'Calendars.ReadWrite.Shared'

  // Mail
  | 'Mail.Read'
  | 'Mail.ReadWrite'
  | 'Mail.Send'
  | 'MailboxSettings.Read'
  | 'MailboxSettings.ReadWrite'

  // Contacts
  | 'Contacts.Read'
  | 'Contacts.ReadWrite'

  // Files
  | 'Files.Read'
  | 'Files.ReadWrite'
  | 'Files.Read.All'
  | 'Files.ReadWrite.All'

  // Teams
  | 'Team.ReadBasic.All'
  | 'TeamMember.Read.All'
  | 'Channel.ReadBasic.All'
  | 'ChannelMessage.Read.All'

  // OneDrive
  | 'Sites.Read.All'
  | 'Sites.ReadWrite.All'
  | 'Sites.Manage.All'

  // Tasks
  | 'Tasks.Read'
  | 'Tasks.ReadWrite'

  // Directory
  | 'Directory.Read.All'
  | 'Directory.ReadWrite.All'
  | 'Directory.AccessAsUser.All';
