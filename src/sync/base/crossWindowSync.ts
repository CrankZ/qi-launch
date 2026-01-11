import { emit, listen } from '@tauri-apps/api/event';
import { load, type Store } from '@tauri-apps/plugin-store';
import { create } from 'zustand';

const setupMap = new Map<string, boolean>();

/**
 * 创建跨窗口同步的 Store
 * 这是一个异步函数，它会先读取本地配置，然后返回一个 Zustand Hook
 */
export async function createSync<V extends object>(
  key: string,
  initialValues: V,
) {
  type StoreState = {
    data: V;
    sync: ((patch: Partial<V>) => Promise<void>) &
      ((key: string, value: unknown) => Promise<void>);
    syncAll: (next: V, persistLocal?: boolean) => Promise<void>;
    reset: () => Promise<void>;
  };

  const shouldPersist = !!key?.trim();

  // 在这里异步加载本地数据
  const preloaded = shouldPersist ? await getLocal(key) : null;

  // 创建 Zustand store
  const useStore = create<StoreState>((set, get) => {
    async function apply(
      payload: Record<string, unknown>,
      persistLocal: boolean = true,
    ): Promise<void> {
      // zustand更新
      set(payload as any);
      // tauri发射
      await emit(`sync:${key}`, payload);
      // tauri本地保存
      await saveLocal(key, shouldPersist && persistLocal, payload);
    }

    async function syncImpl(patchOrKey: any, maybeValue?: any) {
      let next: V;
      if (patchOrKey && typeof patchOrKey === 'object') {
        next = { ...(get().data as any), ...(patchOrKey as any) } as V;
      } else {
        const k = String(patchOrKey);
        next = { ...(get().data as any), [k]: maybeValue } as V;
      }
      await apply({ data: next });
    }

    return {
      data: preloaded ? { ...initialValues, ...(preloaded as V) } : initialValues,
      sync: syncImpl as any,
      syncAll: async (next: V, persistLocal: boolean = true) => {
        await apply({ data: next }, persistLocal);
      },
      reset: async () => {
        await apply({ data: initialValues });
      },
    };
  });

  if (!setupMap.get(key)) {
    setupMap.set(key, true);
    void listen(`sync:${key}`, async (event) => {
      const patch = event.payload as Record<string, unknown>;
      useStore.setState(patch as any);
    });
  }

  // 返回这个 Hook
  return useStore;
}

function getStore(key: string): Promise<Store> {
  return load(`${key}.json`);
}

async function saveLocal(
  key: string,
  persist: boolean,
  payload: Record<string, unknown>,
) {
  if (!persist) return;
  const inst = await getStore(key);
  const v = payload.data as any;
  if (v && typeof v === 'object' && !Array.isArray(v)) {
    for (const nk of Object.keys(v)) {
      await inst.set(nk, v[nk]);
    }
    await inst.save();
  }
}

async function getLocal(key: string): Promise<Record<string, unknown> | null> {
  const inst = await getStore(key);
  const entries = await inst.entries<any>();
  if (!(entries && entries.length > 0)) {
    return null;
  }
  return Object.fromEntries(entries) as Record<string, unknown>;
}
