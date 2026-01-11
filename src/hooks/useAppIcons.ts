import { invoke } from '@tauri-apps/api/core';
import { type } from '@tauri-apps/plugin-os';
import { useEffect, useState } from 'react';
import type { IconSourceInfo } from '../components/form/SettingsForm';
import { useConfigSync } from '../sync/configSync';
import type { IconType } from '../types/icon';
import { sortIcons } from '../utils/appUtils';

interface UseAppIconsResult {
  desktopIcons: IconType[];
  filteredIcons: IconType[];
  loading: boolean;
  loadingTip: string;
  isWindows: boolean;
  availableSources: IconSourceInfo[];
}

/**
 * 管理应用程序图标加载、过滤和平台检测的 Hook
 * @param searchValue 搜索关键词
 * @param hideList 隐藏名单（图标标识符列表）
 */
export function useAppIcons(
  searchValue: string,
  hideList: string[] = [],
): UseAppIconsResult {
  const [desktopIcons, setDesktopIcons] = useState<IconType[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [loadingTip, setLoadingTip] = useState<string>('正在初始化...');
  const [isWindows, setIsWindows] = useState<boolean>(true);
  const [availableSources, setAvailableSources] = useState<IconSourceInfo[]>(
    [],
  );

  const { data: config } = useConfigSync();
  const { orderMode, iconMethod, iconSources } = config;

  // 检测平台并获取可用来源
  useEffect(() => {
    async function initSources() {
      setLoadingTip('正在获取可用来源...');
      try {
        const osType = type();
        const isMac = osType === 'macos';
        const isWin = osType === 'windows'; // 在 Tauri 中，osType 是准确的，不需要 !isMac 判断
        setIsWindows(isWin);

        const sources = await invoke<IconSourceInfo[]>('get_available_sources');
        setAvailableSources(sources);

        if (isMac) {
          const currentSources = useConfigSync.getState().data.iconSources;
          const hasWindowsSources = currentSources.some((s) =>
            [
              'taskbar_pinned',
              'desktop',
              'public_desktop',
              'start_menu',
              'common_start_menu',
              'installed_programs',
              'program_files',
              'program_files_x86',
              'quick_launch',
              'appdata_programs',
              'uwp_apps',
            ].includes(s),
          );

          if (hasWindowsSources) {
            const macDefaultSources = ['applications'];
            await useConfigSync
              .getState()
              .sync('iconSources', macDefaultSources);
            await useConfigSync.getState().sync('iconMethod', 'icns');
          }
        }
      } catch (e) {
        console.warn('初始化来源失败:', e);
      }
    }

    void initSources();
  }, []);

  // 加载图标
  useEffect(() => {
    async function loadIcons() {
      setLoading(true);
      setLoadingTip('正在加载图标...');
      try {
        let icons: IconType[] = [];

        if (iconSources.length === 1) {
          icons = await invoke<IconType[]>('get_icons_from_source', {
            source: iconSources[0],
            method: iconMethod === 'default' ? null : iconMethod,
          });
        } else if (iconSources.length > 1) {
          icons = await invoke<IconType[]>('get_icons_from_multiple_sources', {
            sources: iconSources,
            method: iconMethod === 'default' ? null : iconMethod,
          });
        }

        setDesktopIcons(icons);
      } catch (error) {
        console.error('获取图标失败:', error);
      } finally {
        setLoading(false);
      }
    }

    void loadIcons();
  }, [iconMethod, JSON.stringify(iconSources)]);

  // 搜索过滤和排序逻辑
  function getFilteredIcons(): IconType[] {
    let filtered: IconType[];

    // 1. 基础搜索过滤
    if (!searchValue || searchValue.trim() === '') {
      filtered = [...desktopIcons];
    } else {
      const lowerSearch = searchValue.toLowerCase();
      filtered = desktopIcons.filter((icon) => {
        const name = icon.name || '';
        const target = icon.target_path || '';
        return (
          name.toLowerCase().includes(lowerSearch) ||
          target.toLowerCase().includes(lowerSearch)
        );
      });
    }

    // 2. 平台特定的额外过滤
    if (isWindows && filtered.length > 0) {
      filtered = filtered.filter((icon) => {
        const target = (icon.target_path || '').toLowerCase();
        const file = (icon.file_path || '').toLowerCase();
        const executableExtensions = [
          '.exe',
          '.bat',
          '.cmd',
          '.msi',
          '.com',
          '.lnk',
        ];

        const isExecutable = executableExtensions.some(
          (ext) => target.endsWith(ext) || file.endsWith(ext),
        );

        return isExecutable || icon.file_type === 'UWP App';
      });
    }

    // TODO: 仅UWP才需要过滤这个，就是区分正常应用和UWP应用
    // 3. 排除卸载程序、网页相关以及隐藏名单中的图标
    filtered = filtered.filter((icon) => {
      const name = (icon.name || '').toLowerCase();
      const target = (icon.target_path || '').toLowerCase();
      const file = (icon.file_path || '').toLowerCase();
      const iconKey = `${icon.file_path}-${icon.name}`;

      const isUninstall = name.includes('卸载') || name.includes('uninstall');
      const isWebFile =
        target.endsWith('.url') ||
        file.endsWith('.url') ||
        target.endsWith('.html') ||
        file.endsWith('.html');
      const isWebProtocol =
        target.startsWith('http://') ||
        target.startsWith('https://') ||
        file.startsWith('http://') ||
        file.startsWith('https://');
      const isHidden = hideList.includes(iconKey);

      return !isUninstall && !isWebFile && !isWebProtocol && !isHidden;
    });

    // 4. 排序
    return sortIcons(filtered, orderMode);
  }

  const filteredIcons = getFilteredIcons();

  return {
    desktopIcons,
    filteredIcons,
    loading,
    loadingTip,
    isWindows,
    availableSources,
  };
}
