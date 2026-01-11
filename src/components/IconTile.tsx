import { Button, Dropdown, Flex, Image, Spin, Tooltip, Typography } from 'antd';
import React from 'react';
import type { IconType } from '../types/icon';

const { Text } = Typography;

interface IconTileProps {
  icon: IconType;
  tileSide: number;
  iconImageSide: number;
  showIconName: boolean;
  fontColor?: string;
  isOpening: boolean;
  onClick: (icon: IconType) => void;
  getContextMenu: (icon: IconType) => any;
}

/**
 * 单个图标组件
 * @param icon 图标数据
 * @param tileSide 瓦片边长
 * @param iconImageSide 图标图片边长
 * @param showIconName 是否显示名称
 * @param fontColor 字体颜色
 * @param isOpening 是否正在打开中
 * @param onClick 点击事件回调
 * @param getContextMenu 获取右键菜单配置
 */
export const IconTile: React.FC<IconTileProps> = ({
  icon,
  tileSide,
  iconImageSide,
  showIconName,
  fontColor,
  isOpening,
  onClick,
  getContextMenu,
}) => {
  const iconKey = `${icon.file_path}-${icon.name}`;
  const defaultIcon =
    'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHZpZXdCb3g9IjAgMCA0OCA0OCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjQ4IiBoZWlnaHQ9IjQ4IiBmaWxsPSIjRjVGNUY1Ii8+CjxwYXRoIGQ9Ik0yNCAzNkMzMC42Mjc0IDM2IDM2IDMwLjYyNzQgMzYgMjRDMzYgMTcuMzcyNiAzMC42Mjc0IDEyIDI0IDEyQzE3LjM3MjYgMTIgMTIgMTcuMzcyNiAxMiAyNEMxMiAzMC42Mjc0IDE3LjM3MjYgMzYgMjQgMzZaIiBmaWxsPSIjRDlEOUQ5Ii8+Cjx0ZXh0IHg9IjI0IiB5PSIyOCIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj4/PC90ZXh0Pgo8L3N2Zz4K';

  return (
    <Dropdown
      key={iconKey}
      menu={getContextMenu(icon)}
      trigger={['contextMenu']}
    >
      <Spin spinning={isOpening}>
        <Tooltip title="">
          <Button
            type="text"
            style={{ width: tileSide, height: tileSide }}
            onClick={() => onClick(icon)}
            disabled={isOpening}
          >
            <Flex gap="small" vertical align="center" style={{ width: '100%' }}>
              <Image
                preview={false}
                style={{
                  width: iconImageSide,
                  height: iconImageSide,
                  objectFit: 'contain',
                }}
                src={icon.icon_base64 || defaultIcon}
                fallback={defaultIcon}
              />
              <Text
                ellipsis={{ tooltip: true }}
                title={''}
                style={{
                  display: showIconName ? 'block' : 'none',
                  width: '100%',
                  textAlign: 'center',
                  color: fontColor,
                }}
              >
                {icon.name}
              </Text>
            </Flex>
          </Button>
        </Tooltip>
      </Spin>
    </Dropdown>
  );
};
