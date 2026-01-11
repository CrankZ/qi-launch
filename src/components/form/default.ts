export const DEFAULT_VALUES = {
  // 软件来源
  iconSources: [
    'taskbar_pinned',
    'desktop',
    'public_desktop',
    'start_menu',
    'common_start_menu',
    'uwp_apps',
    'quick_launch',
  ],
  // 主题模式
  themeMode: 'auto',
  // 字体颜色
  fontColor: '#ffffff',
  // 是否显示图标名称
  showIconName: true,
  // 排序模式
  orderMode: 'alphabet_asc',
  // 图标提取方式
  iconMethod: 'high_res',
  // 网格间距
  gridPadding: 10,
  // 图标间距
  tileGap: 20,
  // 单元格尺寸
  tileSide: 210,
  // 图片尺寸
  iconImageSide: 170,
  // 已隐藏的图标
  hideList: [],
};

export const DEFAULT_VALUES_MAC = {
  // 软件来源
  iconSources: ['applications', 'system_applications'],
  // 主题模式
  themeMode: 'auto',
  // 字体颜色
  fontColor: '#ffffff',
  // 是否显示图标名称
  showIconName: true,
  // 排序模式
  orderMode: 'alphabet_asc',
  // 图标提取方式
  iconMethod: 'native',
  // 网格间距
  gridPadding: 0,
  // 图标间距
  tileGap: 8,
  // 单元格尺寸
  tileSide: 140,
  // 图片尺寸
  iconImageSide: 100,
  // 已隐藏的图标
  hideList: [],
};
