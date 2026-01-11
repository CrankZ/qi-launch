import { useConfigSync } from '../sync/configSync.ts';
import type { ConfigItem } from '../sync/types/ConfigItem.ts';

/**
 * 当表单配置发生变化时，同步更新到状态管理中。并且保存到本地
 */
export async function syncValuesConfig(
  changedValues: Partial<ConfigItem>,
): Promise<void> {
  await Promise.all(
    Object.entries(changedValues).map(([key, value]) =>
      useConfigSync.getState().sync(key as keyof ConfigItem, value),
    ),
  );
}
