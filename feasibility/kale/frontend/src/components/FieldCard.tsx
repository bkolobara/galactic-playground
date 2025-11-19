import React from 'react';
import { FieldData, AccountStatus, WorkState, HarvestState } from '../types';
import { styles } from '../styles';

interface FieldCardProps {
  field: FieldData;
  accountStatus: AccountStatus | null;
  publicKey?: string;
  workState: WorkState;
  harvestState: HarvestState;
  onWork: () => void;
  onHarvest: (blockIndex: number) => void;
}

// Utility function to truncate address
const truncateAddress = (address: string): string => {
  if (address.length <= 8) return address;
  return address.slice(0, 4) + '...' + address.slice(-4);
};

export const FieldCard: React.FC<FieldCardProps> = ({
  field,
  accountStatus,
  publicKey,
  workState,
  harvestState,
  onWork,
  onHarvest,
}) => {
  const { blockIndex, pailData, allPails, isCurrent, entropy } = field;

  // Determine what state to show
  const getFieldState = () => {
    if (isCurrent) {
      return 'work';
    } else {
      if (pailData.hasPail && pailData.hasWorked) {
        return 'harvest';
      }
      return 'inactive';
    }
  };

  const fieldState = getFieldState();

  // Check if work is processing (only for current field)
  const isWorkProcessing = isCurrent && (
    workState.status === 'mining' ||
    workState.status === 'preparing' ||
    workState.status === 'signing' ||
    workState.status === 'submitting'
  );

  // Check if harvest is processing (only for non-current fields that can be harvested)
  const isHarvestProcessing = !isCurrent && (
    harvestState.status === 'preparing' ||
    harvestState.status === 'signing' ||
    harvestState.status === 'submitting'
  );

  // Show loading state
  const showLoading = isWorkProcessing || isHarvestProcessing;

  return (
    <div style={styles.actionSection}>
      <h3 style={styles.sectionTitle}>Field {blockIndex}</h3>

      {/* Row 1: Grid of kale emojis from all players */}
      <div style={{
        marginBottom: '12px',
        padding: '10px',
        backgroundColor: '#f5f5f5',
        borderRadius: '6px',
        minHeight: '40px',
      }}>
        {allPails && allPails.length > 0 ? (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
            {allPails.slice(0, 5).map((farmerPail, index) => (
              <span
                key={`${farmerPail.farmerAddress}-${index}`}
                style={{
                  fontSize: '28px',
                  position: 'relative',
                  display: 'inline-block',
                  marginRight: '32px',
                  marginBottom: '10px',
                }}
              >
                🥬
                <sup style={{
                  fontSize: '9px',
                  position: 'absolute',
                  top: '2px',
                  left: '100%',
                  marginLeft: '2px',
                  color: '#666',
                  fontFamily: 'monospace',
                  whiteSpace: 'nowrap',
                }}>
                  {truncateAddress(farmerPail.farmerAddress)}
                </sup>
                <sub style={{
                  fontSize: '11px',
                  position: 'absolute',
                  bottom: '2px',
                  left: '100%',
                  marginLeft: '2px',
                  color: '#333',
                  fontWeight: 'bold',
                  whiteSpace: 'nowrap',
                }}>
                  {farmerPail.pailData.leadingZeros}/10
                </sub>
              </span>
            ))}
            {allPails.length > 5 && (
              <span style={{ fontSize: '12px', color: '#666', alignSelf: 'center' }}>
                +{allPails.length - 5} more kales planted
              </span>
            )}
          </div>
        ) : (
          <span style={{ fontSize: '12px', color: '#999' }}>
            No kales planted in this field yet
          </span>
        )}
      </div>

      {/* Row 2: User's kale status */}
      <div style={{
        marginBottom: '12px',
        padding: '10px',
        backgroundColor: '#e8f5e9',
        borderRadius: '6px',
        fontSize: '13px',
      }}>
        {!pailData.hasPail ? (
          <span>You didn't plant a kale in this field yet.</span>
        ) : fieldState === 'harvest' ? (
          <span>Your kale is ready for harvest 🎉</span>
        ) : (
          <span>
            Your kale: 🥬 {pailData.leadingZeros}/10 quality
            {isCurrent && ', Work on your kale to improve the quality'}
          </span>
        )}
      </div>

      {/* Row 3: Buttons */}
      <div style={{ marginBottom: '8px' }}>
        {fieldState === 'work' && pailData.hasPail && (
          <button
            style={{
              ...styles.button,
              ...(!entropy || isWorkProcessing ? styles.buttonDisabled : {}),
            }}
            onClick={onWork}
            disabled={!entropy || isWorkProcessing}
          >
            {isWorkProcessing
              ? 'Working...'
              : pailData.hasWorked
              ? 'Improve work'
              : 'Work'}
          </button>
        )}

        {fieldState === 'harvest' && (
          <button
            style={{
              ...styles.button,
              ...styles.harvestButton,
              ...(isHarvestProcessing ? styles.buttonDisabled : {}),
            }}
            onClick={() => onHarvest(blockIndex)}
            disabled={isHarvestProcessing}
          >
            {isHarvestProcessing ? 'Harvesting...' : `Harvest field ${blockIndex}`}
          </button>
        )}
      </div>

      {/* Loading messages under buttons */}
      {showLoading && (
        <div style={{
          fontSize: '12px',
          color: '#666',
          fontStyle: 'italic',
          marginTop: '4px',
        }}>
          {workState.status === 'mining' && `Mining... Progress: ${workState.progress?.toFixed(0)}% (Best: ${workState.bestZeros} zeros)`}
          {workState.status === 'preparing' && 'Preparing work transaction...'}
          {workState.status === 'signing' && 'Waiting for signature...'}
          {workState.status === 'submitting' && 'Submitting work transaction...'}
          {harvestState.status === 'preparing' && 'Preparing harvest transaction...'}
          {harvestState.status === 'signing' && 'Waiting for signature...'}
          {harvestState.status === 'submitting' && 'Submitting harvest transaction...'}
        </div>
      )}

      {/* Error messages */}
      {fieldState === 'work' && workState.status === 'error' && (
        <div style={styles.errorBanner}>
          {workState.error}
        </div>
      )}
      {fieldState === 'work' && workState.status === 'failed_to_improve' && !isWorkProcessing && (
        <div style={styles.errorBanner}>
          Failed to make better kale! The new work has fewer leading zeros than before. Try again to find a better hash!
        </div>
      )}
      {fieldState === 'harvest' && harvestState.status === 'error' && (
        <div style={styles.errorBanner}>
          {harvestState.error}
        </div>
      )}
    </div>
  );
};
