import { useState, useEffect } from 'react';
import albedo from '@albedo-link/intent';
import {
  TransactionStep,
  PlantState,
  WorkState,
  HarvestState,
  FundingState,
  TrustlineState,
  BlockInfo,
  PailData
} from '../types';
import { calculateHash, countLeadingZeros } from '../utils/mining';
import { MINING_DURATION } from '../constants';

interface UseTransactionsProps {
  publicKey?: string;
  blockInfo: BlockInfo | null;
  currentPailData: PailData | null;
  accountStatus: any;
  onPlantSuccess?: () => void;
  onWorkSuccess?: () => void;
  onHarvestSuccess?: (blockIndex: number) => void;
  onFundingSuccess?: () => void;
  onTrustlineSuccess?: () => void;
  setBlockInfo?: (info: BlockInfo) => void;
  setCurrentPailData?: (data: PailData | null) => void;
  setAccountStatus?: (status: any) => void;
  loadUserData?: (publicKey: string) => void;
  setIsTransacting?: (value: boolean) => void;
}

export const useTransactions = ({
  publicKey,
  blockInfo,
  currentPailData,
  accountStatus,
  onPlantSuccess,
  onWorkSuccess,
  onHarvestSuccess,
  onFundingSuccess,
  onTrustlineSuccess,
  setBlockInfo,
  setCurrentPailData,
  setAccountStatus,
  loadUserData,
  setIsTransacting,
}: UseTransactionsProps) => {
  const [transactionStep, setTransactionStep] = useState<TransactionStep>('idle');
  const [plantState, setPlantState] = useState<PlantState>({ status: 'idle' });
  const [workState, setWorkState] = useState<WorkState>({ status: 'idle' });
  const [harvestState, setHarvestState] = useState<HarvestState>({ status: 'idle' });
  const [fundingState, setFundingState] = useState<FundingState>({ status: 'idle' });
  const [trustlineState, setTrustlineState] = useState<TrustlineState>({ status: 'idle' });

  // Reset transaction states when field data updates to reflect the completed action
  useEffect(() => {
    // Reset plant state when pail data shows the plant was successful
    if (plantState.status === 'success' && currentPailData?.hasPail) {
      setPlantState({ status: 'idle' });
    }

    // Reset work state when pail data shows the work was successful
    if (workState.status === 'success' && currentPailData?.hasWorked) {
      setWorkState({ status: 'idle' });
    }
  }, [currentPailData, plantState.status, workState.status]);

  const handlePlantTransaction = async () => {
    if (!publicKey) return;

    setIsTransacting?.(true);
    setPlantState({ status: 'preparing' });

    try {
      // Request the prepared transaction from backend
      const prepareResponse = await fetch('/api/plant/prepare', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          publicKey: publicKey,
          amount: '0', // 0 KALE - contract accepts 0 to allow participation without initial tokens
        }),
      });

      const prepareData = await prepareResponse.json();

      if (!prepareResponse.ok) {
        throw new Error(prepareData.error || 'Failed to prepare transaction');
      }

      // Sign the transaction with Albedo
      setPlantState({ status: 'signing' });

      console.log('Sending to Albedo tx():', {
        xdr: prepareData.xdr,
        network: prepareData.network,
        submit: false
      });

      let signResponse;
      try {
        signResponse = await albedo.tx({
          xdr: prepareData.xdr,
          network: prepareData.network,
          submit: false, // We'll submit via our backend
        });
        console.log('Albedo response:', signResponse);
      } catch (albedoError) {
        console.error('Albedo tx() error:', albedoError);
        alert(`Albedo signing error: ${JSON.stringify(albedoError, null, 2)}`);
        throw albedoError;
      }

      // Submit the signed transaction
      setPlantState({ status: 'submitting' });
      const submitResponse = await fetch('/api/plant/submit', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          signedXdr: signResponse.signed_envelope_xdr,
        }),
      });

      const submitData = await submitResponse.json();

      if (!submitResponse.ok) {
        throw new Error(submitData.error || 'Failed to submit transaction');
      }

      setPlantState({ status: 'success', txHash: submitData.hash });
      onPlantSuccess?.();

      // Reload all user data to reflect the latest state (handles new block creation)
      if (publicKey && loadUserData) {
        await loadUserData(publicKey);
      }

    } catch (error) {
      console.error('Plant transaction error:', error);
      let errorMessage = 'Unknown error';

      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'object' && error !== null) {
        errorMessage = JSON.stringify(error, null, 2);
      } else {
        errorMessage = String(error);
      }

      setPlantState({ status: 'error', error: errorMessage });
    } finally {
      setIsTransacting?.(false);
    }
  };

  const handleWorkTransaction = async () => {
    if (!publicKey || !blockInfo) return;

    setIsTransacting?.(true);
    setWorkState({ status: 'mining', progress: 0, bestZeros: 0 });

    // Allow React to render the initial mining state before starting heavy computation
    await new Promise(resolve => setTimeout(resolve, 0));

    try {
      // Mine for 10 seconds with chunked processing to allow UI updates
      const startTime = Date.now();
      let bestHash: Uint8Array | null = null;
      let bestNonce = 0;
      let bestZeros = 0;
      let nonce = 0;

      while (Date.now() - startTime < MINING_DURATION) {
        // Mine in 100ms chunks to allow UI to update between chunks
        const chunkEnd = Date.now() + 100;

        while (Date.now() < chunkEnd && Date.now() - startTime < MINING_DURATION) {
          const hash = calculateHash(
            blockInfo.blockIndex,
            nonce,
            blockInfo.entropy,
            publicKey
          );

          const zeros = countLeadingZeros(hash);

          if (zeros > bestZeros) {
            bestHash = hash;
            bestNonce = nonce;
            bestZeros = zeros;
          }

          nonce++;
        }

        // Update progress after each chunk
        setWorkState({
          status: 'mining',
          progress: Math.min((Date.now() - startTime) / MINING_DURATION * 100, 100),
          bestZeros,
        });

        // Yield to React for rendering
        await new Promise(resolve => setTimeout(resolve, 0));
      }

      if (!bestHash) {
        throw new Error('Failed to find any hash');
      }

      console.log(`Mining complete! Best hash has ${bestZeros} leading zeros (nonce: ${bestNonce})`);

      // If improving work, check if new work has more zeros than previous
      if (currentPailData?.hasWorked && currentPailData.leadingZeros >= bestZeros) {
        console.log(`Failed to improve: new work has ${bestZeros} zeros, previous had ${currentPailData.leadingZeros} zeros`);
        setWorkState({ status: 'failed_to_improve', bestZeros });
        return;
      }

      // Prepare the work transaction
      setWorkState(prev => ({ ...prev, status: 'preparing' }));

      const prepareResponse = await fetch('/api/work/prepare', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          publicKey: publicKey,
          nonce: bestNonce.toString(),
        }),
      });

      const prepareData = await prepareResponse.json();

      if (!prepareResponse.ok) {
        throw new Error(prepareData.error || 'Failed to prepare transaction');
      }

      // Sign the transaction with Albedo
      setWorkState(prev => ({ ...prev, status: 'signing' }));

      console.log('Sending to Albedo tx():', {
        xdr: prepareData.xdr,
        network: prepareData.network,
      });

      const signResponse = await albedo.tx({
        xdr: prepareData.xdr,
        network: prepareData.network,
        submit: false,
      });

      // Submit the signed transaction
      setWorkState(prev => ({ ...prev, status: 'submitting' }));
      const submitResponse = await fetch('/api/work/submit', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          signedXdr: signResponse.signed_envelope_xdr,
        }),
      });

      const submitData = await submitResponse.json();

      if (!submitResponse.ok) {
        throw new Error(submitData.error || 'Failed to submit transaction');
      }

      setWorkState({ status: 'success', txHash: submitData.hash, bestZeros });
      onWorkSuccess?.();

      // Reload all user data to reflect the latest state
      if (publicKey && loadUserData) {
        await loadUserData(publicKey);
      }

    } catch (error) {
      console.error('Work transaction error:', error);
      let errorMessage = 'Unknown error';

      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'object' && error !== null) {
        errorMessage = JSON.stringify(error, null, 2);
      } else {
        errorMessage = String(error);
      }

      setWorkState({ status: 'error', error: errorMessage });
    } finally {
      setIsTransacting?.(false);
    }
  };

  const handleFundAccount = async () => {
    if (!publicKey) return;

    setIsTransacting?.(true);
    setFundingState({ status: 'funding' });

    try {
      const response = await fetch('/api/fund_account', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ publicKey }),
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || 'Failed to fund account');
      }

      setFundingState({ status: 'success' });
      onFundingSuccess?.();

      // Wait briefly for network to process, then reload all data
      await new Promise(resolve => setTimeout(resolve, 2000));
      if (publicKey && loadUserData) {
        await loadUserData(publicKey);
      }

    } catch (error) {
      console.error('Fund account error:', error);
      let errorMessage = 'Unknown error';

      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'object' && error !== null) {
        errorMessage = JSON.stringify(error, null, 2);
      } else {
        errorMessage = String(error);
      }

      setFundingState({ status: 'error', error: errorMessage });
    } finally {
      setIsTransacting?.(false);
    }
  };

  const handleAddTrustline = async () => {
    if (!publicKey) return;

    setIsTransacting?.(true);
    setTrustlineState({ status: 'preparing' });

    try {
      // Prepare the trustline transaction
      const prepareResponse = await fetch('/api/trustline/prepare', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ publicKey }),
      });

      const prepareData = await prepareResponse.json();

      if (!prepareResponse.ok) {
        throw new Error(prepareData.error || 'Failed to prepare trustline transaction');
      }

      // Sign the transaction with Albedo
      setTrustlineState({ status: 'signing' });

      console.log('Sending to Albedo tx():', {
        xdr: prepareData.xdr,
        network: prepareData.network,
      });

      const signResponse = await albedo.tx({
        xdr: prepareData.xdr,
        network: prepareData.network,
        submit: false,
      });

      // Submit the signed transaction
      setTrustlineState({ status: 'submitting' });
      const submitResponse = await fetch('/api/trustline/submit', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          signedXdr: signResponse.signed_envelope_xdr,
        }),
      });

      const submitData = await submitResponse.json();

      if (!submitResponse.ok) {
        throw new Error(submitData.error || 'Failed to submit trustline transaction');
      }

      setTrustlineState({ status: 'success', txHash: submitData.hash });
      onTrustlineSuccess?.();

      // Wait briefly for network to process, then reload all data
      await new Promise(resolve => setTimeout(resolve, 2000));
      if (publicKey && loadUserData) {
        await loadUserData(publicKey);
      }

    } catch (error) {
      console.error('Add trustline error:', error);
      let errorMessage = 'Unknown error';

      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'object' && error !== null) {
        errorMessage = JSON.stringify(error, null, 2);
      } else {
        errorMessage = String(error);
      }

      setTrustlineState({ status: 'error', error: errorMessage });
    } finally {
      setIsTransacting?.(false);
    }
  };

  const handleHarvestTransaction = async (blockIndex: number) => {
    if (!publicKey) return;

    setIsTransacting?.(true);
    setHarvestState({ status: 'preparing' });

    try {
      // Prepare the harvest transaction
      const prepareResponse = await fetch('/api/harvest/prepare', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          publicKey: publicKey,
          blockIndex: blockIndex,
        }),
      });

      const prepareData = await prepareResponse.json();

      if (!prepareResponse.ok) {
        throw new Error(prepareData.error || 'Failed to prepare transaction');
      }

      // Sign the transaction with Albedo
      setHarvestState({ status: 'signing' });

      console.log('Sending to Albedo tx():', {
        xdr: prepareData.xdr,
        network: prepareData.network,
      });

      const signResponse = await albedo.tx({
        xdr: prepareData.xdr,
        network: prepareData.network,
        submit: false,
      });

      // Submit the signed transaction
      setHarvestState({ status: 'submitting' });
      const submitResponse = await fetch('/api/harvest/submit', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          signedXdr: signResponse.signed_envelope_xdr,
        }),
      });

      const submitData = await submitResponse.json();

      if (!submitResponse.ok) {
        throw new Error(submitData.error || 'Failed to submit transaction');
      }

      setHarvestState({ status: 'success', txHash: submitData.hash });
      onHarvestSuccess?.(blockIndex);

      // Reload all user data to reflect the latest state
      if (publicKey && loadUserData) {
        await loadUserData(publicKey);
      }

    } catch (error) {
      console.error('Harvest transaction error:', error);
      let errorMessage = 'Unknown error';

      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'object' && error !== null) {
        errorMessage = JSON.stringify(error, null, 2);
      } else {
        errorMessage = String(error);
      }

      setHarvestState({ status: 'error', error: errorMessage });
    } finally {
      setIsTransacting?.(false);
    }
  };

  return {
    transactionStep,
    plantState,
    workState,
    harvestState,
    fundingState,
    trustlineState,
    setTransactionStep,
    handlePlantTransaction,
    handleWorkTransaction,
    handleFundAccount,
    handleAddTrustline,
    handleHarvestTransaction,
  };
};
