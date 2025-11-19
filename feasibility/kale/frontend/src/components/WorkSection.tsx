import React from 'react';
import { BlockInfo, PailData, WorkState, TransactionStep } from '../types';
import { styles } from '../styles';

interface WorkSectionProps {
  currentPailData: PailData | null;
  blockInfo: BlockInfo | null;
  workState: WorkState;
  transactionStep: TransactionStep;
  onWork: () => void;
}

export const WorkSection: React.FC<WorkSectionProps> = ({
  currentPailData,
  blockInfo,
  workState,
  transactionStep,
  onWork,
}) => {
  return (
    <div style={styles.actionSection}>
      <h3 style={styles.sectionTitle}>Work</h3>
      {currentPailData?.hasWorked && workState.status !== 'failed_to_improve' ? (
        <div style={styles.statusMessage}>
          ✓ Work completed: {currentPailData.leadingZeros} leading zeros
          <br />
          <span style={styles.hint}>You can submit again with MORE zeros to improve</span>
        </div>
      ) : null}
      {workState.status === 'mining' && (
        <div style={styles.statusMessage}>
          Mining... Progress: {workState.progress?.toFixed(0)}%
          <br />
          <span style={styles.hint}>Best zeros found: {workState.bestZeros}</span>
        </div>
      )}
      {workState.status === 'error' && transactionStep === 'idle' && (
        <div style={styles.errorBanner}>
          Mining error: {workState.error}
        </div>
      )}
      <button
        style={{
          ...styles.button,
          ...(!currentPailData?.hasPail || !blockInfo || ['mining', 'preparing', 'signing', 'submitting'].includes(workState.status) ? styles.buttonDisabled : {}),
          ...(workState.status === 'failed_to_improve' ? { backgroundColor: '#f44336' } : {}),
        }}
        onClick={onWork}
        disabled={!currentPailData?.hasPail || !blockInfo || ['mining', 'preparing', 'signing', 'submitting'].includes(workState.status)}
      >
        {['mining', 'preparing', 'signing', 'submitting'].includes(workState.status)
          ? 'WORKING!'
          : workState.status === 'failed_to_improve'
          ? 'FAILED TO MAKE BETTER KALE! TRY AGAIN!'
          : currentPailData?.hasWorked
          ? 'IMPROVE WORK (Mine for 10s)'
          : 'WORK (Mine for 10s)'}
      </button>
      {!currentPailData?.hasPail && (
        <p style={styles.hint}>Plant first to enable work</p>
      )}
      {currentPailData?.hasPail && !blockInfo && (
        <p style={styles.hint}>Block info not available yet - please refresh after planting</p>
      )}
    </div>
  );
};
