/**
 * 类型定义
 * 基于后端 Rust 模型转换为 TypeScript 类型
 */

export interface Ticket {
  id: string; // UUID
  title: string; // 必填，最大长度 255
  description: string | null; // 可选，文本类型
  completed: boolean; // 完成状态，默认 false
  created_at: string; // ISO 8601 格式的时间字符串
  updated_at: string; // ISO 8601 格式的时间字符串
}

export interface Tag {
  id: string; // UUID
  name: string; // 必填，最大长度 50，唯一
  color: string | null; // 可选，十六进制颜色代码（如 "#ff0000"）
  created_at: string; // ISO 8601 格式的时间字符串
}

export interface TicketWithTags extends Ticket {
  tags: Tag[]; // 关联的标签列表
}

export interface CreateTicketRequest {
  title: string;
  description?: string | null;
  tag_ids?: string[] | null; // UUID 数组
}

export interface UpdateTicketRequest {
  title?: string | null;
  description?: string | null;
  completed?: boolean | null;
}

export interface CreateTagRequest {
  name: string;
  color?: string | null;
}

export interface UpdateTagRequest {
  name?: string | null;
  color?: string | null;
}

/**
 * Ticket 查询参数
 */
export interface TicketQuery {
  tag?: string; // 标签 ID (UUID)
  search?: string; // 搜索关键词（标题模糊搜索）
  completed?: boolean; // 完成状态筛选
}

/**
 * 错误响应
 */
export interface ErrorResponse {
  error: string;
  status: number;
}
