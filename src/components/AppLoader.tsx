import { Flex, Spin } from 'antd';
import React from 'react';

interface AppLoaderProps {
  loading: boolean;
  tip: string;
}

export const AppLoader: React.FC<AppLoaderProps> = ({ loading, tip }) => {
  if (!loading) return null;

  return (
    <Flex
      style={{
        position: 'fixed',
        inset: 0,
        // zIndex: 9999,
      }}
      justify="center"
      align="center"
    >
      <Spin tip={tip} size="large">
        <div
          style={{
            padding: 100,
            background: 'rgba(0, 0, 0, 0.05)',
            borderRadius: 4,
          }}
        />
      </Spin>
    </Flex>
  );
};
