import { type } from '@tauri-apps/plugin-os';
import {
  DEFAULT_VALUES,
  DEFAULT_VALUES_MAC,
} from '../components/form/default.ts';
import { createSync } from './base/crossWindowSync.ts';
import type { ConfigItem } from './types/ConfigItem.ts';

const osType = type();
const isMacOS = osType === 'macos';

// 异步创建并导出 Hook。此时 createSync 内部会自动处理异步加载。
export const useConfigSync = await createSync<ConfigItem>('qiConfig', {
  ...(isMacOS ? DEFAULT_VALUES_MAC : DEFAULT_VALUES),
});
