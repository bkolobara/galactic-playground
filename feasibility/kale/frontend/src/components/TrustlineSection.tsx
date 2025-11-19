import React from 'react';
import { AccountStatus, TrustlineState } from '../types';
import { styles } from '../styles';
import { MIN_XLM_BALANCE } from '../constants';

interface TrustlineSectionProps {
  accountStatus: AccountStatus;
  trustlineState: TrustlineState;
  onAddTrustline: () => void;
}

export const TrustlineSection: React.FC<TrustlineSectionProps> = ({
  accountStatus,
  trustlineState,
  onAddTrustline,
}) => {
  // Hide section if trustline exists or was just successfully added
  if (!accountStatus.exists ||
      accountStatus.xlmBalance < MIN_XLM_BALANCE ||
      accountStatus.hasTrustline ||
      trustlineState.status === 'success') {
    return null;
  }

  const isProcessing = trustlineState.status === 'preparing' ||
                       trustlineState.status === 'signing' ||
                       trustlineState.status === 'submitting';

  return (
    <div style={styles.actionSection}>
      <h3 style={styles.sectionTitle}>Add KALE Trustline</h3>
      <p style={styles.hint}>
        You need to add a trustline to KALE token before you can start farming.
      </p>
      <button
        style={{
          ...styles.button,
          ...(isProcessing ? styles.buttonDisabled : {}),
        }}
        onClick={onAddTrustline}
        disabled={isProcessing}
      >
        {isProcessing ? 'Adding trustline...' : 'Add KALE Trustline'}
      </button>
      {trustlineState.status === 'error' && (
        <div style={styles.errorBanner}>
          {trustlineState.error}
        </div>
      )}
    </div>
  );
};
