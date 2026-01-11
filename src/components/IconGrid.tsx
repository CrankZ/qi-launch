import { Empty, Flex } from 'antd';
import React from 'react';
import Slider from 'react-slick';
import 'slick-carousel/slick/slick.css';
import 'slick-carousel/slick/slick-theme.css';
import type { IconType } from '../types/icon';
import { IconTile } from './IconTile';

interface IconGridProps {
  pages: IconType[][];
  tileGap: number;
  gridPadding: number;
  tileSide: number;
  iconImageSide: number;
  showIconName: boolean;
  fontColor?: string;
  openingIcons: Set<string>;
  onIconClick: (icon: IconType) => void;
  getContextMenu: (icon: IconType) => any;
  loading?: boolean;
}

/**
 * 图标网格布局组件（带轮播功能）
 */
export const IconGrid = (props: IconGridProps) => {
  const {
    pages,
    tileGap,
    gridPadding,
    tileSide,
    iconImageSide,
    showIconName,
    fontColor,
    openingIcons,
    onIconClick,
    getContextMenu,
    loading = false,
  } = props;

  const sliderRef = React.useRef<Slider | null>(null);
  const containerRef = React.useRef<HTMLDivElement>(null);
  const isLocked = React.useRef(false);

  React.useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const onWheel = (e: WheelEvent) => {
      if (Math.abs(e.deltaX) < 30 && Math.abs(e.deltaY) < 30) return;
      e.preventDefault();
      if (isLocked.current) return;
      isLocked.current = true;
      e.deltaX + e.deltaY > 0
        ? sliderRef.current?.slickNext()
        : sliderRef.current?.slickPrev();
      setTimeout(() => {
        isLocked.current = false;
      }, 500);
    };

    const onKey = (e: KeyboardEvent) => {
      if (['ArrowRight', 'ArrowDown'].includes(e.key))
        sliderRef.current?.slickNext();
      if (['ArrowLeft', 'ArrowUp'].includes(e.key))
        sliderRef.current?.slickPrev();
    };

    el.addEventListener('wheel', onWheel, { passive: false });
    window.addEventListener('keydown', onKey);
    return () => {
      el.removeEventListener('wheel', onWheel);
      window.removeEventListener('keydown', onKey);
    };
  }, []);

  const sliderSettings = {
    accessibility: true,
    dots: true,
    arrows: false,
    infinite: false,
    slidesToShow: 1,
    slidesToScroll: 1,
    swipe: true,
    draggable: true,
    speed: 500,
    cssEase: 'ease-out',
    appendDots: (dots: React.ReactNode) => (
      <div
        style={{
          position: 'fixed',
          left: 0,
          right: 0,
          bottom: 16,
          display: 'flex',
          justifyContent: 'center',
        }}
      >
        <ul className="slick-dots" style={{ position: 'static', margin: 0 }}>
          {dots}
        </ul>
      </div>
    ),
  };

  if (!loading && pages.length === 0) {
    return (
      <Flex justify="center" align="center" style={{ height: '100%' }}>
        <Empty />
      </Flex>
    );
  }

  return (
    <div
      ref={containerRef}
      style={{
        height: '100%',
        position: 'relative',
        touchAction: 'none',
        overscrollBehavior: 'none',
      }}
    >
      <Slider ref={sliderRef} {...sliderSettings}>
        {pages.map((page, pageIdx) => (
          <div
            key={`page-${pageIdx}-${page[0]?.file_path ?? 'empty'}`}
            style={{ height: '100%' }}
          >
            <Flex
              wrap
              gap={tileGap}
              style={{
                padding: gridPadding,
                margin: 0,
                alignContent: 'center',
                justifyContent: 'flex-start',
                height: '100%',
              }}
            >
              {page.map((icon) => (
                <IconTile
                  key={`${icon.file_path}-${icon.name}`}
                  icon={icon}
                  tileSide={tileSide}
                  iconImageSide={iconImageSide}
                  showIconName={showIconName}
                  fontColor={fontColor}
                  isOpening={openingIcons.has(`${icon.file_path}-${icon.name}`)}
                  onClick={onIconClick}
                  getContextMenu={getContextMenu}
                />
              ))}
            </Flex>
          </div>
        ))}
      </Slider>
    </div>
  );
};

IconGrid.displayName = 'IconGrid';
