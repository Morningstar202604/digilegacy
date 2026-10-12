// 前端共享类型定义（与后端 Rust model 对齐）

export interface Platform {
  id: number;
  name: string;
  category: string;
  subtype: string;
  has_property: boolean;
  has_personality: boolean;
  inheritable: boolean;
  required_documents: string;
  process_description: string;
  estimated_difficulty: string;
  official_policy_url: string;
  estimated_time_days: string;
  notes: string;
}

export interface DigitalAsset {
  id?: number;
  platform_id: number;
  platform_name: string;
  account_identifier: string;
  account_description: string;
  category: 'property' | 'personality' | 'mixed';
  estimated_value: number | null;
  value_currency: string;
  disposal_method: string;
  heir_name: string | null;
  heir_contact: string | null;
  notes: string;
  is_confirmed: boolean;
  created_at: string;
  updated_at: string;
}

export interface AssetRequest {
  platform_id: number;
  account_identifier: string;
  account_description: string;
  category: string;
  estimated_value: number | null;
  disposal_method: string;
  heir_name?: string;
  heir_contact?: string;
  notes?: string;
}

export interface UserProfile {
  id?: number;
  full_name: string;
  phone: string;
  email: string;
  id_last4: string;
  emergency_contact_name: string;
  emergency_contact_phone: string;
  emergency_contact_relation: string;
  heir_primary_name: string;
  heir_primary_phone: string;
  heir_primary_relation: string;
  check_in_interval_days: number;
  last_check_in: string | null;
  dms_enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface DashboardStats {
  total_assets: number;
  property_count: number;
  personality_count: number;
  mixed_count: number;
  confirmed_count: number;
  unconfirmed_count: number;
  total_estimated_value: number;
  disposal_planned_count: number;
  last_check_in: string | null;
  dms_enabled: boolean;
  completion_percentage: number;
}

export interface ApiResponse<T> {
  success: boolean;
  message: string;
  data: T | null;
}

export interface CheckInStatus {
  last_check_in: string | null;
  interval_days: number;
  dms_enabled: boolean;
  next_check_in_due: string | null;
  days_until_due: number | null;
}

export interface DmsNotification {
  stage: number;
  recipient: string;
  subject: string;
  body: string;
  sent_at: string;
  channel: string;
}

export interface DmsEvaluation {
  enabled: boolean;
  overdue: boolean;
  current_stage: number;
  days_overdue: number;
  notifications: DmsNotification[];
}


