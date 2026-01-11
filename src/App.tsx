import { openPath } from '@tauri-apps/plugin-opener';
import { exit } from '@tauri-apps/plugin-process';
import { Layout, Modal, message } from 'antd';
import React, { useEffect, useRef, useState } from 'react';
import './App.css';
import { type } from '@tauri-apps/plugin-os';
import { AppLoader } from './components/AppLoader';
import { SettingsForm } from './components/form/SettingsForm.tsx';
import HeaderBar from './components/HeaderBar';
import { IconGrid } from './components/IconGrid';
import { IconViewModal } from './components/IconViewModal';
import { useAppIcons } from './hooks/useAppIcons';
import { useWindowSize } from './hooks/useWindowSize';
import { useConfigSync } from './sync/configSync.ts';
import type { IconType } from './types/icon';
import { openApplication, revealFile } from './utils/appUtils';

const { Content } = Layout;

/**
 * 应用程序主入口组件
 * 负责整体布局、状态调度及核心交互逻辑
 */
const App: React.FC = () => {
  const [messageApi, contextHolder] = message.useMessage();

  // 状态管理
  const [searchValue, setSearchValue] = useState('');
  const [viewModalVisible, setViewModalVisible] = useState(false);
  const [viewingIcon, setViewingIcon] = useState<IconType | null>(null);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const [openingIcons, setOpeningIcons] = useState<Set<string>>(new Set());

  // 引用
  const contentAreaRef = useRef<HTMLDivElement | null>(null);

  const { data: config, sync: syncConfig } = useConfigSync();
  const {
    gridPadding,
    tileGap,
    showIconName,
    tileSide,
    iconImageSide,
    fontColor,
    hideList,
  } = config;

  const { filteredIcons, loading, loadingTip, availableSources, desktopIcons } =
    useAppIcons(searchValue, hideList);

  const containerSize = useWindowSize(contentAreaRef, [tileSide]);

  // 快捷键处理：Esc 退出应用
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape' && document.hasFocus()) {
        e.preventDefault();
        void exit(0);
      }
    }
    window.addEventListener('keydown', handleKeyDown, true);
    return () => window.removeEventListener('keydown', handleKeyDown, true);
  }, []);

  /**
   * 处理图标点击：启动应用程序
   * @param icon 图标对象
   */
  async function handleIconClick(icon: IconType) {
    const iconKey = `${icon.file_path}-${icon.name}`;
    if (openingIcons.has(iconKey)) return;

    setOpeningIcons((prev) => new Set(prev).add(iconKey));
    try {
      // 保证至少显示 1 秒的加载状态
      await Promise.all([
        openApplication(icon),
        new Promise((resolve) => setTimeout(resolve, 1000)),
      ]);
    } catch (error) {
      console.error('打开应用程序失败:', error);
      messageApi.error(`打开失败: ${icon.name}`);
    } finally {
      setOpeningIcons((prev) => {
        const newSet = new Set(prev);
        newSet.delete(iconKey);
        return newSet;
      });
    }
  }

  /**
   * 获取图标右键菜单配置
   * @param icon 图标对象
   */
  function getContextMenu(icon: IconType) {
    const iconKey = `${icon.file_path}-${icon.name}`;
    const isOpening = openingIcons.has(iconKey);

    return {
      items: [
        {
          key: 'open',
          label: '打开',
          disabled: isOpening,
          onClick: () => handleIconClick(icon),
        },
        {
          key: 'hide',
          label: '隐藏',
          disabled: isOpening,
          onClick: async () => {
            try {
              await syncConfig('hideList', [...hideList, iconKey]);
              messageApi.success(`已隐藏: ${icon.name}`);
            } catch (error) {
              console.error('隐藏图标失败:', error);
              messageApi.error('隐藏失败');
            }
          },
        },
        {
          key: 'openDir',
          label: '打开文件所在的位置',
          disabled: icon.file_type === 'UWP App',
          onClick: () => {
            if (icon.file_type === 'UWP App') return;
            const osType = type();
            const isMacOS = osType === 'macos';
            if (isMacOS) {
              void revealFile(icon.file_path);
            } else {
              const directory = icon.file_path.substring(
                0,
                icon.file_path.lastIndexOf('\\'),
              );
              void openPath(directory);
            }
          },
        },
        {
          key: 'info',
          label: '详细信息',
          onClick: () => {
            setViewingIcon(icon);
            setViewModalVisible(true);
          },
        },
      ],
    };
  }

  /**
   * 计算分页布局
   */
  function calculateLayout() {
    if (!containerSize.width || !containerSize.height) {
      return { pages: [], pageSize: 0 };
    }

    const availableWidth = containerSize.width - gridPadding * 2;
    const availableHeight = containerSize.height - gridPadding * 2;

    // 计算每行每列可容纳的数量
    const cols = Math.max(
      1,
      Math.floor((availableWidth + tileGap) / (tileSide + tileGap)),
    );
    const rows = Math.max(
      1,
      Math.floor((availableHeight + tileGap) / (tileSide + tileGap)),
    );
    const pageSize = cols * rows;

    const pages: IconType[][] = [];
    for (let i = 0; i < filteredIcons.length; i += pageSize) {
      pages.push(filteredIcons.slice(i, i + pageSize));
    }
    return { pages, pageSize };
  }

  const { pages, pageSize } = calculateLayout();

  return (
    <Layout style={{ height: '100vh' }}>
      {contextHolder}

      <HeaderBar
        searchValue={searchValue}
        onSearchChange={setSearchValue}
        onSearchSubmit={setSearchValue}
        onOpenSettings={() => setSettingsVisible(true)}
      />

      <Content style={{ padding: 8, height: '100%', position: 'relative' }}>
        <div ref={contentAreaRef} style={{ height: '100%' }}>
          <AppLoader loading={loading} tip={loadingTip} />

          {/* 只有在容器尺寸准备好后才渲染网格，并使用 pageSize 作为 key 强制重刷 */}
          {containerSize.width > 0 && (
            <IconGrid
              key={`grid-${pageSize}-${filteredIcons.length}`}
              pages={pages}
              tileGap={tileGap}
              gridPadding={gridPadding}
              tileSide={tileSide}
              iconImageSide={iconImageSide}
              showIconName={showIconName}
              fontColor={fontColor}
              openingIcons={openingIcons}
              onIconClick={handleIconClick}
              getContextMenu={getContextMenu}
              loading={loading}
            />
          )}
        </div>
      </Content>

      {/* 图标预览弹窗 */}
      <IconViewModal
        open={viewModalVisible}
        icon={viewingIcon}
        onClose={() => {
          setViewModalVisible(false);
          setViewingIcon(null);
        }}
      />

      {/* 设置弹窗 */}
      <Modal
        mask={{ enabled: true, blur: false }}
        title="设置"
        open={settingsVisible}
        onCancel={() => setSettingsVisible(false)}
        maskClosable={true}
        footer={null}
        centered
        width={720}
      >
        <SettingsForm
          availableSources={availableSources}
          desktopIcons={desktopIcons}
        />
      </Modal>
    </Layout>
  );
};

export default App;
