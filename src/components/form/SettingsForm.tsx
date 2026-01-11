import {
  Button,
  Checkbox,
  ColorPicker,
  Form,
  message,
  Segmented,
  Select,
  Space,
  Tag,
  Typography,
} from 'antd';
import { useEffect } from 'react';
import { ThemeModeEnum } from '../../enums/ThemeModeEnum.ts';
import { useConfigSync } from '../../sync/configSync.ts';
import type { IconType } from '../../types/icon.ts';
import { syncValuesConfig } from '../../utils/formUtils.ts';
import { LabeledSliderInput } from '../LabeledSliderInput.tsx';

export interface IconSourceInfo {
  id: string;
  name: string;
  description: string;
  icon: string;
}

export interface Props {
  availableSources: IconSourceInfo[];
  desktopIcons: IconType[];
}

export const SettingsForm = ({ availableSources, desktopIcons }: Props) => {
  const { data: config, sync: syncConfig } = useConfigSync();
  const [messageApi, contextHolder] = message.useMessage();
  const [form] = Form.useForm();

  // 当 config 发生变化时（如重置或跨窗口同步），更新表单值
  useEffect(() => {
    form.setFieldsValue(config);
  }, [config, form]);

  // 使用 Tauri 返回的可用来源来判定平台 (Tauri 方式)
  const isMac = availableSources.some(
    (s) => s.id === 'applications' || s.id === 'spotlight',
  );

  const windowsOptions = [
    { value: 'smart', label: '智能方式 (ImageList -> PrivateExtractIcons)' },
    { value: 'imagelist', label: '系统图标列表 (最快, 256px)' },
    { value: 'shell', label: 'Shell高级提取 (极快, 256px)' },
    { value: 'high_res', label: 'PrivateExtractIcons (快, 512px)' },
    { value: 'pe_resource', label: 'PE资源提取 (较快, 1024px)' },
    { value: 'thumbnail', label: '缩略图提取 (慢, 1024px)' },
  ];

  const macosOptions = [
    { value: 'native', label: '原生 API (推荐, 完美圆角)' },
    { value: 'icns', label: 'ICNS 提取 (原始文件)' },
  ];

  // horizontal' | 'inline' | 'vertical
  return (
    <Form
      form={form}
      initialValues={config}
      onValuesChange={syncValuesConfig}
      layout="horizontal"
    >
      <Form.Item name="iconSources" label="软件来源">
        <Checkbox.Group>
          {availableSources.map((source) => (
            <Checkbox
              key={source.id}
              value={source.id}
              title={source.description}
            >
              {source.icon} {source.name}
            </Checkbox>
          ))}
        </Checkbox.Group>
      </Form.Item>
      <Form.Item name="themeMode" label="主题模式">
        <Segmented options={Object.values(ThemeModeEnum)} />
      </Form.Item>
      <Form.Item
        name="fontColor"
        label="字体颜色"
        getValueFromEvent={(_, hex) => hex}
        getValueProps={(v) => ({
          value: typeof v === 'string' ? v : '#ffffff',
        })}
      >
        <ColorPicker showText />
      </Form.Item>
      <Form.Item name="showIconName" label="名称显示" valuePropName="checked">
        <Checkbox>显示图标名称</Checkbox>
      </Form.Item>

      <Form.Item name="orderMode" label="排序模式">
        <Select
          popupMatchSelectWidth={false}
          options={[
            { value: 'alphabet_desc', label: '字母降序' },
            { value: 'alphabet_asc', label: '字母升序' },
          ]}
        />
      </Form.Item>

      <Form.Item name="iconMethod" label="图标提取方式">
        <Select
          popupMatchSelectWidth={false}
          options={isMac ? macosOptions : windowsOptions}
        />
      </Form.Item>
      <Form.Item name="gridPadding" label="网格内边距">
        <LabeledSliderInput min={0} max={1000} />
      </Form.Item>

      <Form.Item name="tileGap" label="图标间距">
        <LabeledSliderInput min={0} max={100} />
      </Form.Item>

      <Form.Item name="tileSide" label="单元格尺寸">
        <LabeledSliderInput min={32} max={512} />
      </Form.Item>

      <Form.Item
        noStyle
        shouldUpdate={(prevValues, currentValues) =>
          prevValues.tileSide !== currentValues.tileSide
        }
      >
        {({ getFieldValue }) => (
          <Form.Item name="iconImageSide" label="图片尺寸">
            <LabeledSliderInput min={16} max={getFieldValue('tileSide')} />
          </Form.Item>
        )}
      </Form.Item>

      {(() => {
        const hideList = config.hideList || [];
        const hiddenIcons = desktopIcons.filter((icon) =>
          hideList.includes(`${icon.file_path}-${icon.name}`),
        );

        return (
          <Form.Item label="已隐藏的图标">
            {contextHolder}
            {hiddenIcons.length === 0 ? (
              <Typography.Text type="secondary">
                暂无已隐藏的图标
              </Typography.Text>
            ) : (
              hiddenIcons.length > 0 && (
                <Space wrap>
                  <Button
                    type="link"
                    size="small"
                    onClick={async () => {
                      try {
                        await syncConfig('hideList', []);
                        messageApi.success('已显示所有图标');
                      } catch (error) {
                        console.error('清除隐藏失败:', error);
                        messageApi.error('操作失败');
                      }
                    }}
                  >
                    删除全部隐藏
                  </Button>
                  {hiddenIcons.map((icon) => {
                    const iconKey = `${icon.file_path}-${icon.name}`;
                    return (
                      <Tag
                        key={iconKey}
                        closable
                        onClose={async () => {
                          try {
                            const newHideList = hideList.filter(
                              (key) => key !== iconKey,
                            );
                            await syncConfig('hideList', newHideList);
                            messageApi.success(`已取消隐藏: ${icon.name}`);
                          } catch (error) {
                            console.error('取消隐藏失败:', error);
                            messageApi.error('操作失败');
                          }
                        }}
                      >
                        {icon.name}
                      </Tag>
                    );
                  })}
                </Space>
              )
            )}
          </Form.Item>
        );
      })()}
    </Form>
  );
};
