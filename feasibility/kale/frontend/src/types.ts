export type TransactionStep = 'idle' | 'signing' | 'complete';

export interface AuthState {
  status: 'idle' | 'loading' | 'success' | 'error';
  publicKey?: string;
  error?: string;
}

export interface PlantState {
  status: 'idle' | 'preparing' | 'signing' | 'submitting' | 'success' | 'error';
  txHash?: string;
  error?: string;
}

export interface WorkState {
  status: 'idle' | 'mining' | 'preparing' | 'signing' | 'submitting' | 'success' | 'error' | 'failed_to_improve';
  txHash?: string;
  error?: string;
  progress?: number;
  bestZeros?: number;
}

export interface HarvestState {
  status: 'idle' | 'preparing' | 'signing' | 'submitting' | 'success' | 'error';
  txHash?: string;
  error?: string;
}

export interface AccountStatus {
  exists: boolean;
  xlmBalance: number; // in stroops
  hasTrustline: boolean;
}

export interface FundingState {
  status: 'idle' | 'funding' | 'success' | 'error';
  error?: string;
}

export interface TrustlineState {
  status: 'idle' | 'preparing' | 'signing' | 'submitting' | 'success' | 'error';
  txHash?: string;
  error?: string;
}

export interface BlockInfo {
  blockIndex: number;
  entropy: string;
}

export interface PailData {
  hasPail: boolean;
  hasWorked: boolean;
  leadingZeros: number;
}

export interface FarmerPailData {
  farmerAddress: string;
  pailData: PailData;
}

export interface FieldData {
  blockIndex: number;
  pailData: PailData; // Current user's pail data
  allPails: FarmerPailData[]; // All farmers' pail data
  isCurrent: boolean;
  entropy?: string;
}
