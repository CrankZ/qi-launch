export interface ConfigItem {
  // 图标源
  iconSources: string[];
  // 主题模式
  themeMode: string;
  // 字体颜色
  fontColor: string;
  // 是否显示图标名称
  showIconName: boolean;
  // 排序模式
  orderMode: string;
  // 图标提取方式
  iconMethod: string;
  // 网格内边距
  gridPadding: number;
  // 图标间距
  tileGap: number;
  // 单元格尺寸
  tileSide: number;
  // 图片尺寸
  iconImageSide: number;
  // 隐藏图标列表
  hideList: string[];
}
