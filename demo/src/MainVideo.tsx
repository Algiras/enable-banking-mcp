import React from 'react';
import { AbsoluteFill } from 'remotion';
import { TransitionSeries, linearTiming, springTiming } from '@remotion/transitions';
import { fade } from '@remotion/transitions/fade';
import { slide } from '@remotion/transitions/slide';
import { FPS } from './theme';

import { SceneHero }         from './scenes/SceneHero';
import { SceneUserStory }    from './scenes/SceneUserStory';
import { SceneArchitecture } from './scenes/SceneArchitecture';
import { SceneAuth }         from './scenes/SceneAuth';
import { SceneAccountData }  from './scenes/SceneAccountData';
import { SceneTransactions } from './scenes/SceneTransactions';
import { SceneSpending }     from './scenes/SceneSpending';
import { ScenePayments }     from './scenes/ScenePayments';
import { SceneMcpUI }        from './scenes/SceneMcpUI';
import { SceneAllTools }     from './scenes/SceneAllTools';
import { SceneOutro }        from './scenes/SceneOutro';

const S = {
  hero:         5,
  userStory:    7,
  architecture: 6,
  auth:         7,
  accountData:  6,
  transactions: 6,
  spending:     6,
  payments:     6,
  mcpUI:        6,
  allTools:     6,
  outro:        7,
};

const TRANS = 20;
const NUM_TRANSITIONS = Object.keys(S).length - 1;
export const TOTAL_FRAMES =
  Object.values(S).reduce((a, b) => a + b, 0) * FPS - NUM_TRANSITIONS * TRANS;

const fadeTiming  = linearTiming({ durationInFrames: TRANS });
const slideTiming = springTiming({ config: { damping: 200 }, durationInFrames: TRANS });

export const MainVideo: React.FC = () => {
  return (
    <AbsoluteFill style={{ background: '#07080f' }}>
      <TransitionSeries>
        <TransitionSeries.Sequence durationInFrames={S.hero         * FPS}><SceneHero /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={fadeTiming} />

        <TransitionSeries.Sequence durationInFrames={S.userStory    * FPS}><SceneUserStory /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={slide({ direction: 'from-right' })} timing={slideTiming} />

        <TransitionSeries.Sequence durationInFrames={S.architecture * FPS}><SceneArchitecture /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={fadeTiming} />

        <TransitionSeries.Sequence durationInFrames={S.auth         * FPS}><SceneAuth /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={slide({ direction: 'from-right' })} timing={slideTiming} />

        <TransitionSeries.Sequence durationInFrames={S.accountData  * FPS}><SceneAccountData /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={slide({ direction: 'from-right' })} timing={slideTiming} />

        <TransitionSeries.Sequence durationInFrames={S.transactions * FPS}><SceneTransactions /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={slide({ direction: 'from-right' })} timing={slideTiming} />

        <TransitionSeries.Sequence durationInFrames={S.spending     * FPS}><SceneSpending /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={slide({ direction: 'from-right' })} timing={slideTiming} />

        <TransitionSeries.Sequence durationInFrames={S.payments     * FPS}><ScenePayments /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={fadeTiming} />

        <TransitionSeries.Sequence durationInFrames={S.mcpUI        * FPS}><SceneMcpUI /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={slide({ direction: 'from-right' })} timing={slideTiming} />

        <TransitionSeries.Sequence durationInFrames={S.allTools     * FPS}><SceneAllTools /></TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={fadeTiming} />

        <TransitionSeries.Sequence durationInFrames={S.outro        * FPS}><SceneOutro /></TransitionSeries.Sequence>
      </TransitionSeries>
    </AbsoluteFill>
  );
};
