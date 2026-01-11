import { Image, Modal, Typography } from 'antd';
import type { IconType } from '../types/icon';

export interface IconViewModalProps {
  // 关键变量：弹窗可见性
  open: boolean;
  // 关键变量：当前预览的图标
  icon: IconType | null;
  // 方法：关闭弹窗
  onClose: () => void;
}

// 图标查看弹窗（显示大图与详细信息）
export const IconViewModal = ({ open, icon, onClose }: IconViewModalProps) => {
  const dpr = window.devicePixelRatio || 1;
  const logicalWidth = icon ? icon.icon_width / dpr : 0;
  const logicalHeight = icon ? icon.icon_height / dpr : 0;

  return (
    <Modal
      mask={{ enabled: true, blur: false }}
      title={icon ? `查看图标 - ${icon.name}` : '查看图标'}
      open={open}
      onCancel={onClose}
      maskClosable={true}
      footer={null}
      width={Math.max(400, logicalWidth + 100)}
      centered
    >
      {icon && (
        <div style={{ textAlign: 'center', padding: '20px' }}>
          <Image
            src={icon.icon_base64}
            alt={icon.name}
            width={logicalWidth}
            height={logicalHeight}
            style={{
              maxWidth: '100%',
              maxHeight: '70vh',
              objectFit: 'contain',
              // 针对高分屏优化显示质量
              imageRendering: 'auto',
            }}
            preview={true}
            fallback="data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHZpZXdCb3g9IjAgMCA0OCA0OCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjQ4IiBoZWlnaHQ9IjQ4IiBmaWxsPSIjRjVGNUY1Ii8+CjxwYXRoIGQ9Ik0yNCAzNkMzMC42Mjc0IDM2IDM2IDMwLjYyNzQgMzYgMjRDMzYgMTcuMzcyNiAzMC42Mjc0IDEyIDI0IDEyQzE3LjM3MjYgMTIgMTIgMTcuMzcyNiAxMiAyNEMxMiAzMC42Mjc0IDE3LjM3MjYgMzYgMjQgMzZaIiBmaWxsPSIjRDlEOUQ5Ii8+Cjx0ZXh0IHg9IjI0IiB5PSIyOCIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj4/PC90ZXh0Pgo8L3N2Zz4K"
          />
          <div style={{ marginTop: '16px' }}>
            <Typography.Text strong>{icon.name}</Typography.Text>
            <br />
            <Typography.Text type="secondary">
              尺寸: {icon.icon_width} × {icon.icon_height} 像素
            </Typography.Text>
            <br />
            <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
              目标: {icon.target_path}
            </Typography.Text>
            <br />
            <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
              位置: {icon.file_path}
            </Typography.Text>
            {icon.source_name && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  软件来源: {String(icon.source_name)}
                </Typography.Text>
              </>
            )}
            {icon.file_type && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  文件类型: {String(icon.file_type)}
                </Typography.Text>
              </>
            )}
            {icon.file_size && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  文件大小: {(Number(icon.file_size) / 1024).toFixed(2)} KB
                </Typography.Text>
              </>
            )}
            {icon.created_time && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  创建时间: {String(icon.created_time)}
                </Typography.Text>
              </>
            )}
            {icon.modified_time && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  修改时间: {String(icon.modified_time)}
                </Typography.Text>
              </>
            )}
            {icon.accessed_time && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  访问时间: {String(icon.accessed_time)}
                </Typography.Text>
              </>
            )}
            {icon.description && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  描述: {String(icon.description)}
                </Typography.Text>
              </>
            )}
            {icon.arguments && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  启动参数: {String(icon.arguments)}
                </Typography.Text>
              </>
            )}
            {icon.working_directory && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  工作目录: {String(icon.working_directory)}
                </Typography.Text>
              </>
            )}
            {icon.hotkey && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  快捷键: {String(icon.hotkey)}
                </Typography.Text>
              </>
            )}
            {icon.show_command && (
              <>
                <br />
                <Typography.Text type="secondary" style={{ fontSize: '12px' }}>
                  运行方式: {String(icon.show_command)}
                </Typography.Text>
              </>
            )}
          </div>
        </div>
      )}
    </Modal>
  );
};
