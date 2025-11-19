import { useState, useEffect } from 'react';
import albedo from '@albedo-link/intent';
import { AuthState } from '../types';
import { STORAGE_KEY } from '../constants';

export const useAuth = (onAuthSuccess?: (publicKey: string) => void) => {
  const [authState, setAuthState] = useState<AuthState>({ status: 'idle' });

  // Check localStorage on startup
  useEffect(() => {
    const storedPubkey = localStorage.getItem(STORAGE_KEY);
    if (storedPubkey) {
      setAuthState({ status: 'success', publicKey: storedPubkey });
      onAuthSuccess?.(storedPubkey);
    }
  }, []);

  // Connect wallet with Albedo
  const handleConnectWallet = async () => {
    setAuthState({ status: 'loading' });

    try {
      // Request public key from Albedo
      const response = await albedo.publicKey({
        token: crypto.randomUUID(),
      });

      const publicKey = response.pubkey;

      // Store in localStorage
      localStorage.setItem(STORAGE_KEY, publicKey);

      setAuthState({ status: 'success', publicKey });

      // Notify parent component
      onAuthSuccess?.(publicKey);

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setAuthState({ status: 'error', error: errorMessage });

      // Notify backend of error
      await fetch('/api/pubkey', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ error: errorMessage }),
      });
    }
  };

  // Logout and clear session
  const handleLogout = () => {
    localStorage.removeItem(STORAGE_KEY);
    setAuthState({ status: 'idle' });
  };

  return {
    authState,
    handleConnectWallet,
    handleLogout,
  };
};
