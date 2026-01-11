import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import type { IconType } from '../types/icon';

/**
 * 打开应用程序
 * 使用 file_path（位置）而不是 target_path（目标），这样可以保留快捷方式的参数
 * @param icon 图标对象
 */
export async function openApplication(icon: IconType): Promise<void> {
  let filePath = icon.file_path;

  // 如果是 UWP 应用，使用 shell:AppsFolder 协议启动
  if (icon.file_type === 'UWP App') {
    filePath = `shell:AppsFolder\\${icon.target_path}`;
  }

  try {
    return await openPath(filePath);
  } catch (error) {
    console.error('打开应用程序失败:', error);
    throw error;
  }
}

/**
 * 图标排序函数
 * @param icons 图标列表
 * @param mode 排序模式：alphabet_asc (字母升序), alphabet_desc (字母降序)
 */
export function sortIcons(icons: IconType[], mode: string): IconType[] {
  return [...icons].sort((a, b) => {
    // 获取首字母进行比较
    const firstCharA = a.name.charAt(0);
    const firstCharB = b.name.charAt(0);

    if (mode === 'alphabet_asc') {
      return firstCharA.localeCompare(firstCharB, 'zh-CN');
    } else if (mode === 'alphabet_desc') {
      return firstCharB.localeCompare(firstCharA, 'zh-CN');
    }
    return 0;
  });
}

export async function revealFile(path: string): Promise<void> {
  await invoke<void>('reveal_file', { path });
}
