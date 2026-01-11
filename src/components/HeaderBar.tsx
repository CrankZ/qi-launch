import {
  BorderOutlined,
  CloseOutlined,
  FullscreenOutlined,
  MinusOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { exit } from '@tauri-apps/plugin-process';
import { Button, Flex, Input, Tooltip } from 'antd';

export interface HeaderBarProps {
  // 关键变量：搜索关键词
  searchValue: string;
  // 方法：当输入变化时回调
  onSearchChange: (value: string) => void;
  // 方法：当提交搜索时回调
  onSearchSubmit: (value: string) => void;
  // 方法：打开设置弹窗
  onOpenSettings: () => void;
}

const HeaderBar = ({
  searchValue,
  onSearchChange,
  onSearchSubmit,
  onOpenSettings,
}: HeaderBarProps) => {
  return (
    <div
      data-tauri-drag-region
      style={{
        position: 'sticky',
        top: 0,
        zIndex: 1,
        width: '100%',
        display: 'flex',
        alignItems: 'center',
        alignContent: 'center',
        justifyContent: 'flex-start',
        flexWrap: 'nowrap',
        height: 'auto',
        padding: '0 0',
        paddingBottom: '2px',
        backgroundColor: 'transparent',
        gap: '8px',
        userSelect: 'none',
        cursor: 'default',
      }}
    >
      <div data-tauri-drag-region style={{ flex: 1 }} />
      <Flex gap="small" style={{ width: '50%', paddingTop: '4px' }}>
        <Input
          value={searchValue}
          variant="filled"
          placeholder="搜索..."
          allowClear
          onPressEnter={(e) =>
            onSearchSubmit((e.target as HTMLInputElement).value)
          }
          onChange={(e) => onSearchChange(e.target.value)}
          style={{ backgroundColor: 'transparent', userSelect: 'text' }}
        />
        <Tooltip title="设置">
          <Button
            icon={<SettingOutlined />}
            type="text"
            htmlType="button"
            onClick={onOpenSettings}
          />
        </Tooltip>
      </Flex>
      <Flex
        data-tauri-drag-region
        style={{ flex: 1, justifyContent: 'flex-end' }}
      >
        <Tooltip title="最小化">
          <Button
            icon={<MinusOutlined />}
            type="text"
            onClick={() => {
              void getCurrentWebviewWindow().minimize();
            }}
          />
        </Tooltip>
        <Tooltip title="最大化">
          <Button
            icon={<BorderOutlined />}
            type="text"
            onClick={async () => {
              const win = getCurrentWebviewWindow();
              const isMaximized = await win.isMaximized();
              const isFullscreen = await win.isFullscreen();

              if (isFullscreen) {
                await win.setFullscreen(false);
              }

              if (isMaximized) {
                await win.unmaximize();
              } else {
                await win.maximize();
              }
            }}
          />
        </Tooltip>
        <Tooltip title="全屏">
          <Button
            icon={<FullscreenOutlined />}
            type="text"
            onClick={async () => {
              const win = getCurrentWebviewWindow();
              const isFullscreen = await win.isFullscreen();
              const isMaximized = await win.isMaximized();

              if (isFullscreen) {
                await win.setFullscreen(false);
              } else {
                // 进入全屏前彻底清理最大化状态，防止工作区限制导致底部空白
                if (isMaximized) {
                  await win.unmaximize();
                }
                await win.setFullscreen(true);
              }
            }}
          />
        </Tooltip>
        <Tooltip title="关闭">
          <Button
            icon={<CloseOutlined />}
            type="text"
            onClick={() => {
              void exit(0);
            }}
          />
        </Tooltip>
      </Flex>
    </div>
  );
};
export default HeaderBar;
