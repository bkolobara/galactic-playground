import React from 'react';
import { AccountStatus, FundingState } from '../types';
import { styles } from '../styles';
import { MIN_XLM_BALANCE } from '../constants';

interface FundingSectionProps {
  accountStatus: AccountStatus;
  fundingState: FundingState;
  onFund: () => void;
}

export const FundingSection: React.FC<FundingSectionProps> = ({
  accountStatus,
  fundingState,
  onFund,
}) => {
  // Hide section if account is funded or was just successfully funded
  if ((accountStatus.exists && accountStatus.xlmBalance >= MIN_XLM_BALANCE) ||
      fundingState.status === 'success') {
    return null;
  }

  return (
    <div style={styles.actionSection}>
      <h3 style={styles.sectionTitle}>Fund Account</h3>
      {!accountStatus.exists ? (
        <p style={styles.hint}>Account doesn't exist yet. Fund it with Friendbot to get started.</p>
      ) : (
        <p style={styles.hint}>
          Balance: {(accountStatus.xlmBalance / 10000000).toFixed(2)} XLM (need at least 50 XLM)
        </p>
      )}
      <button
        style={{
          ...styles.button,
          ...(fundingState.status === 'funding' ? styles.buttonDisabled : {}),
        }}
        onClick={onFund}
        disabled={fundingState.status === 'funding'}
      >
        {fundingState.status === 'funding' ? 'Funding...' : 'Fund with Friendbot'}
      </button>
      {fundingState.status === 'error' && (
        <div style={styles.errorBanner}>
          {fundingState.error}
        </div>
      )}
    </div>
  );
};
