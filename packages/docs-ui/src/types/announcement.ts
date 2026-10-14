export type CloudStatusIndicator = 'none' | 'minor' | 'major' | 'critical' | 'maintenance';

export type CloudStatus = {
  indicator: CloudStatusIndicator;
  description: string;
};

export type CloudAnnouncement = {
  id: string;
  title: string;
  subtitle?: string;
  link_url?: string;
  published_at?: string;
  created_at: string;
  type: 'feature' | 'incident' | 'version';
};
