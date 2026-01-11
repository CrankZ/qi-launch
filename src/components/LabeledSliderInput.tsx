import { Flex, InputNumber, Slider } from 'antd';
import { useEffect, useRef, useState } from 'react';

export interface LabeledSliderInputProps {
  value?: number;
  min: number;
  max: number;
  onChange?: (value: number) => void;
}

export const LabeledSliderInput = ({
  value = 0,
  min,
  max,
  onChange,
}: LabeledSliderInputProps) => {
  const [localValue, setLocalValue] = useState<number>(value);
  const latestRef = useRef<number>(value);

  useEffect(() => {
    setLocalValue(value);
    latestRef.current = value;
  }, [value]);

  function handleSliderChange(val: number) {
    setLocalValue(val);
    latestRef.current = val;
  }

  function handleSliderChangeComplete() {
    const v = Math.min(max, Math.max(min, latestRef.current));
    onChange?.(v);
  }

  function handleInputChange(val: string | number | null) {
    const num = Number(val);
    const v = Number.isFinite(num) ? Math.min(max, Math.max(min, num)) : min;
    setLocalValue(v);
    latestRef.current = v;
    onChange?.(v);
  }

  return (
    <Flex style={{ width: '100%' }} align="center" gap="small">
      <Slider
        min={min}
        max={max}
        style={{ flex: 1, minWidth: 0 }}
        value={localValue}
        onChange={handleSliderChange}
        onChangeComplete={handleSliderChangeComplete}
      />
      <InputNumber
        min={min}
        max={max}
        style={{ width: 80, flexShrink: 0 }}
        value={localValue}
        onChange={handleInputChange}
      />
    </Flex>
  );
};
