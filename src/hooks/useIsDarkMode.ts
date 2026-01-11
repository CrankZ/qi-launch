import { getCurrentWindow, type Theme } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';
import { useConfigSync } from '../sync/configSync';

/**
 * 获取当前生效的暗色模式状态
 * 逻辑：
 * 1. 如果 config.themeMode 为 'dark'，返回 true
 * 2. 如果 config.themeMode 为 'light'，返回 false
 * 3. 如果 config.themeMode 为 'auto'，返回系统当前的主题状态
 */
export const useIsDarkMode = () => {
  const { data: config } = useConfigSync();
  const [systemTheme, setSystemTheme] = useState<Theme | null>(null);

  useEffect(() => {
    const win = getCurrentWindow();

    // 初始化系统主题
    win.theme().then(setSystemTheme);

    // 监听系统主题变化
    let unlisten: (() => void) | null = null;
    win
      .onThemeChanged(({ payload }) => {
        setSystemTheme(payload as Theme);
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  if (config.themeMode === 'dark') {
    return true;
  }
  if (config.themeMode === 'light') {
    return false;
  }

  // auto 模式下返回系统主题
  return systemTheme === 'dark';
};
