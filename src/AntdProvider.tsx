import { setTheme } from '@tauri-apps/api/app';
import type { Theme } from '@tauri-apps/api/window';
import { ConfigProvider, Layout, theme } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import React, { useEffect } from 'react';
import { useIsDarkMode } from './hooks/useIsDarkMode.ts';
import { useConfigSync } from './sync/configSync.ts';

/**
 * 主题提供器组件：统一管理自动/亮色/暗色
 */
const AntdProvider = ({ children }: { children: React.ReactNode }) => {
  const { data: config } = useConfigSync();
  const isDarkMode = useIsDarkMode();

  useEffect(() => {
    if (config.themeMode === 'auto') {
      void setTheme(null);
    } else {
      void setTheme(config.themeMode as Theme);
    }
  }, [config.themeMode]);

  useEffect(() => {
    document.documentElement.style.colorScheme = isDarkMode ? 'dark' : 'light';
  }, [isDarkMode]);

  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
        components: {
          Layout: {
            bodyBg: 'transparent',
            footerBg: 'transparent',
            headerBg: 'transparent',
            lightSiderBg: 'transparent',
            siderBg: 'transparent',
          },
        },
      }}
    >
      <Layout>{children}</Layout>
    </ConfigProvider>
  );
};

export default AntdProvider;
