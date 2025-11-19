import React from 'react';
import {
  TransactionStep,
  PlantState,
  WorkState,
  HarvestState,
  TrustlineState,
  BlockInfo,
} from '../types';
import { styles } from '../styles';

interface TransactionProgressProps {
  transactionStep: TransactionStep;
  plantState: PlantState;
  workState: WorkState;
  harvestState: HarvestState;
  trustlineState: TrustlineState;
  blockInfo: BlockInfo | null;
  onPlantRetry: () => void;
  onWorkRetry: () => void;
  onBackToFarm: () => void;
}

export const TransactionProgress: React.FC<TransactionProgressProps> = ({
  transactionStep,
  plantState,
  workState,
  harvestState,
  trustlineState,
  blockInfo,
  onPlantRetry,
  onWorkRetry,
  onBackToFarm,
}) => {
  // Transaction Signing Progress
  if (transactionStep === 'signing') {
    return (
      <div style={styles.statusContainer}>
        {/* Plant transaction states */}
        {plantState.status === 'preparing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Preparing transaction...</p>
            <p style={styles.hint}>Building and simulating your plant transaction</p>
          </>
        )}

        {plantState.status === 'signing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Waiting for signature...</p>
            <p style={styles.hint}>Please sign the transaction in the Albedo popup</p>
          </>
        )}

        {plantState.status === 'submitting' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Submitting transaction...</p>
            <p style={styles.hint}>Sending to the Stellar network</p>
          </>
        )}

        {plantState.status === 'error' && (
          <>
            <div style={styles.errorIcon}>✗</div>
            <p style={styles.errorMessage}>Transaction Failed</p>
            <div style={styles.errorContainer}>
              <p style={styles.error}>{plantState.error}</p>
              {plantState.error?.includes('trustline') && (
                <div style={styles.trustlineHelp}>
                  <p style={styles.helpTitle}>How to add a KALE trustline:</p>
                  <ol style={styles.helpList}>
                    <li>Visit <a href="https://albedo.link" target="_blank" rel="noopener noreferrer" style={styles.link}>Albedo</a></li>
                    <li>Click on "Manage Assets" or "Add Asset"</li>
                    <li>Enter Asset Code: <code style={styles.code}>KALE</code></li>
                    <li>Enter Issuer: <code style={styles.code}>GCHPTWXMT3HYF4RLZHWBNRF4MPXLTJ76ISHMSYIWCCDXWUYOQG5MR2AB</code></li>
                    <li>Confirm and add the trustline</li>
                    <li>Return here and try again</li>
                  </ol>
                </div>
              )}
            </div>
            <button style={styles.button} onClick={onPlantRetry}>
              Try Again
            </button>
          </>
        )}

        {/* Work transaction states */}
        {workState.status === 'mining' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Mining...</p>
            <p style={styles.hint}>Finding the best hash (Progress: {workState.progress?.toFixed(0)}%)</p>
            <p style={styles.hint}>Best zeros found: {workState.bestZeros}</p>
          </>
        )}

        {workState.status === 'preparing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Preparing work transaction...</p>
            <p style={styles.hint}>Building and simulating your work transaction</p>
          </>
        )}

        {workState.status === 'signing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Waiting for signature...</p>
            <p style={styles.hint}>Please sign the transaction in the Albedo popup</p>
          </>
        )}

        {workState.status === 'submitting' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Submitting work transaction...</p>
            <p style={styles.hint}>Sending to the Stellar network</p>
          </>
        )}

        {workState.status === 'error' && (
          <>
            <div style={styles.errorIcon}>✗</div>
            <p style={styles.errorMessage}>Work Transaction Failed</p>
            <div style={styles.errorContainer}>
              <p style={styles.error}>{workState.error}</p>
            </div>
            <button style={styles.button} onClick={onWorkRetry}>
              Try Again
            </button>
          </>
        )}

        {/* Harvest transaction states */}
        {harvestState.status === 'preparing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Preparing harvest transaction...</p>
            <p style={styles.hint}>Building and simulating your harvest transaction</p>
          </>
        )}

        {harvestState.status === 'signing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Waiting for signature...</p>
            <p style={styles.hint}>Please sign the transaction in the Albedo popup</p>
          </>
        )}

        {harvestState.status === 'submitting' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Submitting harvest transaction...</p>
            <p style={styles.hint}>Sending to the Stellar network</p>
          </>
        )}

        {harvestState.status === 'error' && (
          <>
            <div style={styles.errorIcon}>✗</div>
            <p style={styles.errorMessage}>Harvest Transaction Failed</p>
            <div style={styles.errorContainer}>
              <p style={styles.error}>{harvestState.error}</p>
            </div>
            <button style={styles.button} onClick={onBackToFarm}>
              Back
            </button>
          </>
        )}

        {/* Trustline transaction states */}
        {trustlineState.status === 'preparing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Preparing trustline transaction...</p>
            <p style={styles.hint}>Building trustline transaction for KALE</p>
          </>
        )}

        {trustlineState.status === 'signing' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Waiting for signature...</p>
            <p style={styles.hint}>Please sign the transaction in the Albedo popup</p>
          </>
        )}

        {trustlineState.status === 'submitting' && (
          <>
            <div style={styles.spinner}></div>
            <p style={styles.message}>Submitting trustline transaction...</p>
            <p style={styles.hint}>Sending to the Stellar network</p>
          </>
        )}

        {trustlineState.status === 'error' && (
          <>
            <div style={styles.errorIcon}>✗</div>
            <p style={styles.errorMessage}>Trustline Transaction Failed</p>
            <div style={styles.errorContainer}>
              <p style={styles.error}>{trustlineState.error}</p>
            </div>
            <button style={styles.button} onClick={onBackToFarm}>
              Back
            </button>
          </>
        )}
      </div>
    );
  }

  // Transaction Complete
  if (transactionStep === 'complete' && (plantState.status === 'success' || workState.status === 'success' || harvestState.status === 'success' || trustlineState.status === 'success')) {
    return (
      <div style={styles.statusContainer}>
        <div style={styles.successIcon}>✓</div>
        <p style={styles.successMessage}>Transaction Submitted!</p>
        {plantState.status === 'success' && (
          <>
            <p style={styles.publicKey}>Hash: {plantState.txHash}</p>
            <p style={styles.hint}>Plant transaction completed!</p>
          </>
        )}
        {workState.status === 'success' && (
          <>
            {workState.txHash && <p style={styles.publicKey}>Hash: {workState.txHash}</p>}
            <p style={styles.message}>Best zeros achieved: {workState.bestZeros}</p>
            <p style={styles.hint}>
              {workState.txHash
                ? 'Work transaction completed! Wait for the next block to start, then refresh to harvest your rewards.'
                : 'Work completed for block ' + blockInfo?.blockIndex + '! Wait for the next block to start, then refresh to harvest your rewards.'}
            </p>
          </>
        )}
        {harvestState.status === 'success' && (
          <>
            <p style={styles.publicKey}>Hash: {harvestState.txHash}</p>
            <p style={styles.hint}>Harvest transaction completed! Your rewards have been claimed.</p>
          </>
        )}
        {trustlineState.status === 'success' && (
          <>
            <p style={styles.publicKey}>Hash: {trustlineState.txHash}</p>
            <p style={styles.hint}>KALE trustline added successfully! You can now plant seeds.</p>
          </>
        )}
        <button style={styles.button} onClick={onBackToFarm}>
          Back to Farm
        </button>
      </div>
    );
  }

  return null;
};
