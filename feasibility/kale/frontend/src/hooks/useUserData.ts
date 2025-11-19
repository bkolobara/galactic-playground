import { useState, useEffect, useCallback } from 'react';
import { AccountStatus, BlockInfo, PailData, FieldData, FarmerPailData } from '../types';

const KNOWN_FARMERS_KEY = 'kale_known_farmers';

// Load known farmers from localStorage
const loadKnownFarmers = (): Set<string> => {
  try {
    const stored = localStorage.getItem(KNOWN_FARMERS_KEY);
    if (stored) {
      return new Set(JSON.parse(stored));
    }
  } catch (error) {
    console.error('Error loading known farmers:', error);
  }
  return new Set();
};

// Save known farmers to localStorage
const saveKnownFarmers = (farmers: Set<string>) => {
  try {
    localStorage.setItem(KNOWN_FARMERS_KEY, JSON.stringify(Array.from(farmers)));
  } catch (error) {
    console.error('Error saving known farmers:', error);
  }
};

export const useUserData = () => {
  const [accountStatus, setAccountStatus] = useState<AccountStatus | null>(null);
  const [blockInfo, setBlockInfo] = useState<BlockInfo | null>(null);
  const [currentPailData, setCurrentPailData] = useState<PailData | null>(null);
  const [harvestableBlocks, setHarvestableBlocks] = useState<number[]>([]);
  const [hasPlanted, setHasPlanted] = useState<boolean>(false);
  const [fields, setFields] = useState<FieldData[]>([]);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [knownFarmers, setKnownFarmers] = useState<Set<string>>(loadKnownFarmers());
  const [isTransacting, setIsTransacting] = useState<boolean>(false);

  // Helper function to check if fields data has actually changed
  const areFieldsEqual = (fields1: FieldData[], fields2: FieldData[]): boolean => {
    if (fields1.length !== fields2.length) return false;

    return fields1.every((f1, i) => {
      const f2 = fields2[i];
      if (!f2) return false;

      // Compare critical fields
      if (f1.blockIndex !== f2.blockIndex) return false;
      if (f1.isCurrent !== f2.isCurrent) return false;
      if (f1.pailData.hasPail !== f2.pailData.hasPail) return false;
      if (f1.pailData.hasWorked !== f2.pailData.hasWorked) return false;
      if (f1.pailData.leadingZeros !== f2.pailData.leadingZeros) return false;

      // Compare allPails length and content
      if (f1.allPails.length !== f2.allPails.length) return false;
      const allPailsEqual = f1.allPails.every((p1, j) => {
        const p2 = f2.allPails[j];
        if (!p2) return false;
        return p1.farmerAddress === p2.farmerAddress &&
               p1.pailData.hasPail === p2.pailData.hasPail &&
               p1.pailData.hasWorked === p2.pailData.hasWorked &&
               p1.pailData.leadingZeros === p2.pailData.leadingZeros;
      });
      if (!allPailsEqual) return false;

      return true;
    });
  };

  // Load fields data (last 5 blocks)
  const loadFieldsData = useCallback(async (userPublicKey: string) => {
    try {
      // Try to get block info - it might not exist if no one has planted yet
      const blockInfoResponse = await fetch('/api/block_info');
      if (!blockInfoResponse.ok) {
        console.log('Block info not available yet');
        return;
      }

      const blockInfoData = await blockInfoResponse.json();
      if (typeof blockInfoData.blockIndex !== 'number') {
        return;
      }

      // Only update blockInfo if it actually changed
      setBlockInfo(prev => {
        if (prev && prev.blockIndex === blockInfoData.blockIndex && prev.entropy === blockInfoData.entropy) {
          return prev; // Keep same reference to avoid re-render
        }
        return blockInfoData;
      });

      // Fetch data for the last 5 blocks
      const fieldsData: FieldData[] = [];
      const currentBlockIndex = blockInfoData.blockIndex;

      for (let i = 0; i < 5; i++) {
        const blockIndex = currentBlockIndex - i;
        if (blockIndex < 0) break;

        // Fetch user's pail data
        const pailDataResponse = await fetch('/api/pail_data', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            publicKey: userPublicKey,
            blockIndex,
          }),
        });
        const pailData = await pailDataResponse.json();

        // Fetch pail data for all known farmers
        let allPails: FarmerPailData[] = [];
        const farmersToQuery = Array.from(knownFarmers);

        if (farmersToQuery.length > 0) {
          try {
            const allFarmersResponse = await fetch('/api/all_farmers', {
              method: 'POST',
              headers: {
                'Content-Type': 'application/json',
              },
              body: JSON.stringify({
                blockIndex,
                farmerAddresses: farmersToQuery,
              }),
            });

            if (allFarmersResponse.ok) {
              const allFarmersData = await allFarmersResponse.json();
              allPails = allFarmersData.farmers.map((farmer: any) => ({
                farmerAddress: farmer.farmerAddress,
                pailData: {
                  hasPail: farmer.hasPail,
                  hasWorked: farmer.hasWorked,
                  leadingZeros: farmer.leadingZeros,
                },
              }));
            }
          } catch (error) {
            console.error(`Error fetching all farmers for block ${blockIndex}:`, error);
          }
        }

        fieldsData.push({
          blockIndex,
          pailData,
          allPails,
          isCurrent: blockIndex === currentBlockIndex,
          entropy: blockIndex === currentBlockIndex ? blockInfoData.entropy : undefined,
        });
      }

      // Only update fields if data actually changed (prevents unnecessary re-renders)
      setFields(prev => {
        if (areFieldsEqual(prev, fieldsData)) {
          return prev; // Keep same reference to avoid re-render
        }
        return fieldsData;
      });

      // Update current pail data for backward compatibility
      if (fieldsData.length > 0) {
        setCurrentPailData(fieldsData[0].pailData);
      }

      // Update harvestable blocks
      const harvestable = fieldsData
        .filter(f => !f.isCurrent && f.pailData.hasPail && f.pailData.hasWorked)
        .map(f => f.blockIndex);
      setHarvestableBlocks(harvestable);
    } catch (error) {
      console.error('Error loading fields data:', error);
    }
  }, [knownFarmers]);

  // Add a farmer to the known farmers list
  const addKnownFarmer = useCallback((farmerAddress: string) => {
    setKnownFarmers(prev => {
      const updated = new Set(prev);
      updated.add(farmerAddress);
      saveKnownFarmers(updated);
      return updated;
    });
  }, []);

  // Load user data after authentication
  const loadUserData = async (userPublicKey: string) => {
    setPublicKey(userPublicKey);

    // Add the current user to known farmers
    addKnownFarmer(userPublicKey);

    try {
      // Send the public key back to the Rust backend
      await fetch('/api/pubkey', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ pubkey: userPublicKey }),
      });

      // Check account status (balance and trustline)
      const statusResponse = await fetch('/api/account_status', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ publicKey: userPublicKey }),
      });
      const statusData = await statusResponse.json();
      setAccountStatus(statusData);

      // Check if the user has already planted in the current block
      const checkPlantedResponse = await fetch('/api/check_planted', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ publicKey: userPublicKey }),
      });

      const checkPlantedData = await checkPlantedResponse.json();
      setHasPlanted(checkPlantedData.has_planted);

      // Load fields data
      await loadFieldsData(userPublicKey);
    } catch (error) {
      console.error('Error loading user data:', error);
    }
  };

  const resetUserData = () => {
    setAccountStatus(null);
    setBlockInfo(null);
    setCurrentPailData(null);
    setHarvestableBlocks([]);
    setHasPlanted(false);
    setFields([]);
    setPublicKey(null);
  };

  // Polling effect - refetch fields data every 5 seconds (paused during transactions)
  useEffect(() => {
    if (!publicKey || isTransacting) return;

    const intervalId = setInterval(() => {
      loadFieldsData(publicKey);
    }, 5000);

    return () => clearInterval(intervalId);
  }, [publicKey, knownFarmers, loadFieldsData, isTransacting]);

  return {
    accountStatus,
    blockInfo,
    currentPailData,
    harvestableBlocks,
    hasPlanted,
    fields,
    loadUserData,
    resetUserData,
    setAccountStatus,
    setBlockInfo,
    setCurrentPailData,
    setHarvestableBlocks,
    setHasPlanted,
    isTransacting,
    setIsTransacting,
  };
};
