// 图标相关类型定义

export interface IconType {
  name: string;
  icon_base64: string;
  target_path: string;
  file_path: string;
  icon_width: number;
  icon_height: number;
  icon_source_path?: string;
  icon_source_index?: number;

  // 时间信息
  created_time?: string; // 创建时间
  modified_time?: string; // 修改时间
  accessed_time?: string; // 访问时间

  // 文件信息
  file_size?: number; // 文件大小（字节）
  file_type?: string; // 文件类型/扩展名

  // 快捷方式专属信息
  description?: string; // 描述/备注
  arguments?: string; // 启动参数
  working_directory?: string; // 工作目录
  hotkey?: string; // 快捷键
  show_command?: string; // 运行方式（正常、最小化、最大化）
  source_name?: string; // 软件来源名称（如：用户桌面、开始菜单等）
}

export interface IconMethod {
  id: string;
  name: string;
  description: string;
  maxSize: number;
}
