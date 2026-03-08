import React from 'react';
import { Composition } from 'remotion';
import { MainVideo, TOTAL_FRAMES } from './MainVideo';
import { FPS } from './theme';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="EnableBankingDemo"
        component={MainVideo}
        durationInFrames={TOTAL_FRAMES}
        fps={FPS}
        width={1280}
        height={720}
      />
    </>
  );
};
