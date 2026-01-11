import { type RefObject, useEffect, useState } from 'react';

interface WindowSize {
  width: number;
  height: number;
}

/**
 * 监听元素或窗口大小变化的 Hook
 * @param ref 目标元素的引用
 * @param dependencies 触发重新测量的依赖项
 */
export function useWindowSize(
  ref: RefObject<HTMLDivElement | null>,
  dependencies: any[] = [],
): WindowSize {
  const [size, setSize] = useState<WindowSize>({
    width: 0,
    height: 0,
  });

  useEffect(() => {
    function measure() {
      const el = ref.current;
      if (el) {
        setSize({
          width: el.clientWidth,
          height: el.clientHeight,
        });
      }
    }

    measure();
    window.addEventListener('resize', measure);
    return () => window.removeEventListener('resize', measure);
  }, [ref, ...dependencies]);

  return size;
}
