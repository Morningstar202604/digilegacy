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

export interface PlatformExtended extends Platform {
  name_en: string;
  data_source: string;
  inheritance_form_url: string;
  monetization_path: string;
  legal_template_ids: string;
  risk_warnings: string;
  tags: string;
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
  has_credential: boolean;
  is_confirmed: boolean;
  created_at: string;
  updated_at: string;
}

export interface SecurityStatus {
  has_master_password: boolean;
  unlocked: boolean;
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
  emergency_contact_email: string;
  heir_primary_name: string;
  heir_primary_phone: string;
  heir_primary_relation: string;
  heir_primary_email: string;
  check_in_interval_days: number;
  last_check_in: string | null;
  dms_enabled: boolean;
  created_at: string;
  updated_at: string;
}

/** 报告里的平台政策摘要（后端 `get_policy_briefs_db` 产出，导出与预览同源）。 */
export interface PlatformPolicyBrief {
  platform_id: number;
  name: string;
  required_documents: string;
  process_description: string;
  estimated_difficulty: string;
  estimated_time_days: string;
  official_policy_url: string;
  monetization_path: string;
  risk_warnings: string;
  tags: string;
  has_source: boolean;
}

export interface DashboardStats {
  total_assets: number;
  property_count: number;
  personality_count: number;
  mixed_count: number;
  /** 已设定处置方式的资产数 */
  disposal_planned_count: number;
  /** 处置为「继承」且已填继承人的资产数（其他处置视为满足） */
  heir_assigned_count: number;
  /** 账号标识已填且不是解密占位符的资产数 */
  identifier_ready_count: number;
  /** 触发链是否可用：DMS 启用且至少有一条可送达通道 */
  trigger_ready: boolean;
  /** 实际可用通道：smtp / heartbeat / none */
  trigger_channel: 'smtp' | 'heartbeat' | 'none';
  total_estimated_value: number;
  last_check_in: string | null;
  dms_enabled: boolean;
  /** 四项检查各 25% 的加权结果 */
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
  /** 最近 5 次确认时间（新的在前） */
  recent_check_ins?: string[];
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

/** 只读 DMS 状态：界面渲染用，不触发投递。 */
export interface DmsStatus {
  enabled: boolean;
  overdue: boolean;
  current_stage: number;
  days_overdue: number;
}


