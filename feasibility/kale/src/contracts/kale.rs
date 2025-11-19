use anyhow::{Context, Result};
use stellar_xdr::curr::{Int128Parts, ReadXdr, ScAddress, ScVal, WriteXdr};
use stellar_strkey::Strkey;

use crate::rpc::SorobanRpc;

/// KALE contract client
pub struct Kale {
    rpc: SorobanRpc,
}

impl Kale {
    /// Create a new KALE contract client
    ///
    /// # Arguments
    /// * `rpc_url` - The Soroban RPC endpoint URL
    /// * `contract_address` - The KALE contract address
    /// * `network_passphrase` - The network passphrase
    pub fn new(rpc_url: &str, contract_address: &str, network_passphrase: &str) -> Result<Self> {
        let rpc = SorobanRpc::new(rpc_url, contract_address, network_passphrase)?;
        Ok(Self { rpc })
    }

    /// Get the current farm block index from the KALE contract
    ///
    /// Reads the "FarmIndex" value from the contract's instance storage.
    pub async fn get_block_index(&self) -> Result<u32> {
        let instance = self.rpc.get_contract_instance().await?;
        let value = SorobanRpc::parse_instance_storage_value(&instance, "FarmIndex")?;

        if let ScVal::U32(index) = value {
            Ok(index)
        } else {
            anyhow::bail!("FarmIndex is not a U32 value: {:?}", value)
        }
    }

    /// Build, simulate, and prepare a plant transaction
    ///
    /// # Arguments
    /// * `farmer_public_key` - The farmer's Stellar public key
    /// * `amount` - The amount of KALE to stake (in stroops, 7 decimal places)
    ///
    /// Returns the transaction XDR (base64) ready for signing
    pub async fn prepare_plant_transaction(
        &self,
        farmer_public_key: &str,
        amount: i128,
    ) -> Result<String> {
        // KALE token details (from the contract)
        const KALE_ASSET_CODE: &str = "KALE";
        const KALE_ISSUER: &str = "GCHPTWXMT3HYF4RLZHWBNRF4MPXLTJ76ISHMSYIWCCDXWUYOQG5MR2AB";

        // Check if the farmer has a trustline to the KALE token
        let (has_trustline, _balance) = self
            .rpc
            .check_trustline_and_balance(farmer_public_key, KALE_ASSET_CODE, KALE_ISSUER)
            .await?;

        if !has_trustline {
            anyhow::bail!(
                "Account does not have a trustline to {}:{}. \
                Please add the trustline using a Stellar wallet like Albedo, Freighter, or Stellar Laboratory. \
                Visit https://albedo.link or https://laboratory.stellar.org/#explorer to add the trustline.",
                KALE_ASSET_CODE,
                KALE_ISSUER
            );
        }
        // Parse farmer address to ScAddress
        let farmer_strkey = Strkey::from_string(farmer_public_key)?;
        let farmer_address = match farmer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                ScAddress::Account(stellar_xdr::curr::AccountId(
                    stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
                        stellar_xdr::curr::Uint256(pk.0)
                    )
                ))
            }
            _ => anyhow::bail!("Invalid farmer public key type"),
        };

        // Build function arguments for plant(farmer: Address, amount: i128)
        let args = vec![
            ScVal::Address(farmer_address),
            ScVal::I128(Int128Parts {
                hi: (amount >> 64) as i64,
                lo: (amount & 0xFFFFFFFFFFFFFFFF) as u64,
            }),
        ];

        // Build the transaction
        let mut transaction = self.rpc
            .build_invoke_transaction(farmer_public_key, "plant", args)
            .await?;

        // Simulate to get fees and footprint
        let simulation = self.rpc.simulate_transaction(&transaction).await?;

        // Check for simulation errors
        if let Some(error) = &simulation.error {
            anyhow::bail!("Transaction simulation failed: {}", error);
        }

        // Apply simulation results
        transaction = self.rpc.apply_simulation_to_transaction(transaction, &simulation)?;

        // Wrap the transaction in a TransactionV1Envelope (required for signing)
        // Albedo expects a TransactionEnvelope, not a raw Transaction
        let tx_envelope = stellar_xdr::curr::TransactionEnvelope::Tx(
            stellar_xdr::curr::TransactionV1Envelope {
                tx: transaction,
                signatures: stellar_xdr::curr::VecM::default(), // Empty signatures
            },
        );

        // Convert the envelope to XDR base64 for signing
        let tx_xdr = tx_envelope.to_xdr_base64(stellar_xdr::curr::Limits::none())?;

        Ok(tx_xdr)
    }

    /// Submit a signed plant transaction
    ///
    /// # Arguments
    /// * `signed_tx_xdr` - The signed transaction XDR (base64)
    ///
    /// Returns the transaction hash
    pub async fn submit_plant_transaction(&self, signed_tx_xdr: &str) -> Result<String> {
        self.rpc.submit_transaction(signed_tx_xdr).await
    }

    /// Get the network passphrase (needed for Albedo signing)
    pub fn network_passphrase(&self) -> &str {
        self.rpc.network_passphrase()
    }

    /// Check if a farmer has planted in the current block
    ///
    /// # Arguments
    /// * `farmer_public_key` - The farmer's Stellar public key
    ///
    /// Returns true if the farmer has a Pail entry for the current block
    pub async fn has_planted(&self, farmer_public_key: &str) -> Result<bool> {
        let block_index = self.get_block_index().await?;

        // Parse farmer address
        let farmer_strkey = Strkey::from_string(farmer_public_key)?;
        let farmer_address = match farmer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                ScAddress::Account(stellar_xdr::curr::AccountId(
                    stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
                        stellar_xdr::curr::Uint256(pk.0)
                    )
                ))
            }
            _ => anyhow::bail!("Invalid farmer public key type"),
        };

        // Build the Pail storage key: Pail(farmer, block_index)
        let pail_key = stellar_xdr::curr::LedgerKey::ContractData(
            stellar_xdr::curr::LedgerKeyContractData {
                contract: ScAddress::Contract(stellar_xdr::curr::Hash(self.rpc.contract_id().0.clone())),
                key: ScVal::Vec(Some(stellar_xdr::curr::ScVec(vec![
                    ScVal::Symbol(stellar_xdr::curr::ScSymbol("Pail".try_into()?)),
                    ScVal::Address(farmer_address),
                    ScVal::U32(block_index),
                ].try_into()?))),
                durability: stellar_xdr::curr::ContractDataDurability::Temporary,
            },
        );

        // Try to fetch the Pail entry
        let entry = self.rpc.get_ledger_entry(pail_key).await?;
        Ok(entry.is_some())
    }

    /// Get the current block information (index and entropy)
    ///
    /// Returns (block_index, Option<block_entropy>)
    /// Entropy is None if nobody has planted in the current block yet
    pub async fn get_block_info(&self) -> Result<(u32, Option<[u8; 32]>)> {
        // Get the farm index from instance storage (this always exists)
        let instance = self.rpc.get_contract_instance().await?;
        let index_value = SorobanRpc::parse_instance_storage_value(&instance, "FarmIndex")?;
        let block_index = if let ScVal::U32(index) = index_value {
            index
        } else {
            anyhow::bail!("FarmIndex is not a U32 value: {:?}", index_value)
        };

        // IMPORTANT: Get the Block at the specific index from temporary storage, NOT FarmBlock!
        // The contract's work function uses: get_block(&env, index).entropy
        // Note: The Block entry is only created when the first person plants in the block
        let block_key = stellar_xdr::curr::LedgerKey::ContractData(
            stellar_xdr::curr::LedgerKeyContractData {
                contract: ScAddress::Contract(stellar_xdr::curr::Hash(self.rpc.contract_id().0.clone())),
                key: ScVal::Vec(Some(stellar_xdr::curr::ScVec(vec![
                    ScVal::Symbol(stellar_xdr::curr::ScSymbol("Block".try_into()?)),
                    ScVal::U32(block_index),
                ].try_into()?))),
                durability: stellar_xdr::curr::ContractDataDurability::Temporary,
            },
        );

        let block_entry = self.rpc.get_ledger_entry(block_key).await?;

        // If the Block entry doesn't exist, nobody has planted yet
        let entropy = match block_entry {
            None => None,
            Some(entry) => {
                // Decode the LedgerEntryData from base64 XDR
                let entry_data = stellar_xdr::curr::LedgerEntryData::from_xdr_base64(
                    &entry.xdr,
                    stellar_xdr::curr::Limits::none(),
                )?;

                // Parse the Block struct to extract entropy
                if let stellar_xdr::curr::LedgerEntryData::ContractData(contract_data) = entry_data {
                    if let ScVal::Map(Some(map)) = contract_data.val {
                        let mut entropy_bytes = None;
                        for entry in map.iter() {
                            if let ScVal::Symbol(ref sym) = entry.key {
                                if sym.to_utf8_string_lossy() == "entropy" {
                                    if let ScVal::Bytes(ref bytes) = entry.val {
                                        if bytes.len() == 32 {
                                            let mut arr = [0u8; 32];
                                            arr.copy_from_slice(&bytes[..]);
                                            entropy_bytes = Some(arr);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        entropy_bytes
                    } else {
                        anyhow::bail!("Block is not a Map: {:?}", contract_data.val)
                    }
                } else {
                    anyhow::bail!("Ledger entry is not ContractData")
                }
            }
        };

        Ok((block_index, entropy))
    }

    /// Calculate the work hash for a given nonce
    ///
    /// # Arguments
    /// * `farmer_public_key` - The farmer's Stellar public key
    /// * `nonce` - The nonce to use for hash calculation
    ///
    /// Returns the calculated hash
    pub async fn calculate_work_hash(
        &self,
        farmer_public_key: &str,
        nonce: u64,
    ) -> Result<[u8; 32]> {
        use sha3::{Digest, Keccak256};

        let (block_index, entropy_opt) = self.get_block_info().await?;
        let entropy = entropy_opt
            .context("Cannot calculate work hash - nobody has planted in this block yet")?;

        // Parse farmer address to get raw bytes for XDR encoding
        let farmer_strkey = Strkey::from_string(farmer_public_key)?;
        let farmer_address_scval = match farmer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                ScAddress::Account(stellar_xdr::curr::AccountId(
                    stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
                        stellar_xdr::curr::Uint256(pk.0)
                    )
                ))
            }
            _ => anyhow::bail!("Invalid farmer public key type"),
        };

        // Encode ScAddress to XDR and get last 32 bytes (matches contract logic)
        // NOTE: The contract encodes Address directly (which is ScAddress in XDR), NOT wrapped in ScVal
        use stellar_xdr::curr::WriteXdr;
        let farmer_xdr = farmer_address_scval.to_xdr(stellar_xdr::curr::Limits::none())?;
        let farmer_bytes = &farmer_xdr[farmer_xdr.len() - 32..];

        // Build the 76-byte input: block_index (4) + nonce (8) + entropy (32) + farmer (32)
        let mut hash_input = [0u8; 76];
        hash_input[0..4].copy_from_slice(&block_index.to_be_bytes());
        hash_input[4..12].copy_from_slice(&nonce.to_be_bytes());
        hash_input[12..44].copy_from_slice(&entropy);
        hash_input[44..76].copy_from_slice(farmer_bytes);

        // Calculate Keccak256
        let mut hasher = Keccak256::new();
        hasher.update(&hash_input);
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    /// Build, simulate, and prepare a work transaction
    ///
    /// # Arguments
    /// * `farmer_public_key` - The farmer's Stellar public key
    /// * `nonce` - The nonce used to generate the hash
    ///
    /// Returns the transaction XDR (base64) ready for signing
    pub async fn prepare_work_transaction(
        &self,
        farmer_public_key: &str,
        nonce: u64,
    ) -> Result<String> {
        // Calculate the hash using current block info
        let hash = self.calculate_work_hash(farmer_public_key, nonce).await?;
        // Parse farmer address to ScAddress
        let farmer_strkey = Strkey::from_string(farmer_public_key)?;
        let farmer_address = match farmer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                ScAddress::Account(stellar_xdr::curr::AccountId(
                    stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
                        stellar_xdr::curr::Uint256(pk.0)
                    )
                ))
            }
            _ => anyhow::bail!("Invalid farmer public key type"),
        };

        // Build function arguments for work(farmer: Address, hash: BytesN<32>, nonce: u64)
        // Note: BytesN<32> is represented as ScVal::Bytes with ScBytes containing the fixed 32 bytes
        let args = vec![
            ScVal::Address(farmer_address),
            ScVal::Bytes(stellar_xdr::curr::ScBytes(hash.to_vec().try_into()?)),
            ScVal::U64(nonce),
        ];

        // Build the transaction
        let mut transaction = self.rpc
            .build_invoke_transaction(farmer_public_key, "work", args)
            .await?;

        // Simulate to get fees and footprint
        let simulation = self.rpc.simulate_transaction(&transaction).await?;

        // Check for simulation errors
        if let Some(error) = &simulation.error {
            anyhow::bail!("Transaction simulation failed: {}", error);
        }

        // Apply simulation results
        transaction = self.rpc.apply_simulation_to_transaction(transaction, &simulation)?;

        // Wrap the transaction in a TransactionV1Envelope (required for signing)
        let tx_envelope = stellar_xdr::curr::TransactionEnvelope::Tx(
            stellar_xdr::curr::TransactionV1Envelope {
                tx: transaction,
                signatures: stellar_xdr::curr::VecM::default(),
            },
        );

        // Convert the envelope to XDR base64 for signing
        let tx_xdr = tx_envelope.to_xdr_base64(stellar_xdr::curr::Limits::none())?;

        Ok(tx_xdr)
    }

    /// Submit a signed work transaction
    ///
    /// # Arguments
    /// * `signed_tx_xdr` - The signed transaction XDR (base64)
    ///
    /// Returns the transaction hash
    pub async fn submit_work_transaction(&self, signed_tx_xdr: &str) -> Result<String> {
        self.rpc.submit_transaction(signed_tx_xdr).await
    }

    /// Get the Pail data for a farmer in a specific block
    ///
    /// # Arguments
    /// * `farmer_public_key` - The farmer's Stellar public key
    /// * `block_index` - The block index to query
    ///
    /// Returns (has_pail, has_worked, leading_zeros) tuple
    pub async fn get_pail_data(&self, farmer_public_key: &str, block_index: u32) -> Result<(bool, bool, u32)> {
        // Parse farmer address
        let farmer_strkey = Strkey::from_string(farmer_public_key)?;
        let farmer_address = match farmer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                ScAddress::Account(stellar_xdr::curr::AccountId(
                    stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
                        stellar_xdr::curr::Uint256(pk.0)
                    )
                ))
            }
            _ => anyhow::bail!("Invalid farmer public key type"),
        };

        // Build the Pail storage key
        let pail_key = stellar_xdr::curr::LedgerKey::ContractData(
            stellar_xdr::curr::LedgerKeyContractData {
                contract: ScAddress::Contract(stellar_xdr::curr::Hash(self.rpc.contract_id().0.clone())),
                key: ScVal::Vec(Some(stellar_xdr::curr::ScVec(vec![
                    ScVal::Symbol(stellar_xdr::curr::ScSymbol("Pail".try_into()?)),
                    ScVal::Address(farmer_address),
                    ScVal::U32(block_index),
                ].try_into()?))),
                durability: stellar_xdr::curr::ContractDataDurability::Temporary,
            },
        );

        // Try to fetch the Pail entry
        let entry = self.rpc.get_ledger_entry(pail_key).await?;

        match entry {
            None => Ok((false, false, 0)),
            Some(ledger_entry_result) => {
                // Decode the LedgerEntryData from base64 XDR
                let entry_data = stellar_xdr::curr::LedgerEntryData::from_xdr_base64(
                    &ledger_entry_result.xdr,
                    stellar_xdr::curr::Limits::none(),
                )?;

                // Parse the Pail struct
                if let stellar_xdr::curr::LedgerEntryData::ContractData(contract_data) = entry_data {
                    if let ScVal::Map(Some(map)) = contract_data.val {
                        // Extract zeros field (Option<u32>)
                        let mut zeros_value = None;

                        for entry in map.iter() {
                            if let ScVal::Symbol(ref sym) = entry.key {
                                if sym.to_utf8_string_lossy() == "zeros" {
                                    // In the Pail struct, zeros is Option<u32>
                                    // When Some(value), Soroban serializes it directly as U32(value)
                                    // When None, the field might not be present or be represented differently
                                    match &entry.val {
                                        ScVal::U32(zeros) => {
                                            zeros_value = Some(*zeros);
                                        }
                                        ScVal::Vec(Some(vec)) if vec.is_empty() => {
                                            // None case - empty Vec represents None
                                            zeros_value = None;
                                        }
                                        ScVal::Vec(Some(vec)) => {
                                            // Vec with one element
                                            if let Some(ScVal::U32(zeros)) = vec.first() {
                                                zeros_value = Some(*zeros);
                                            }
                                        }
                                        _ => {}
                                    }
                                    break;
                                }
                            }
                        }

                        let has_worked = zeros_value.is_some();
                        let zeros = zeros_value.unwrap_or(0);
                        Ok((true, has_worked, zeros))
                    } else {
                        anyhow::bail!("Pail value is not a Map")
                    }
                } else {
                    anyhow::bail!("Ledger entry is not ContractData")
                }
            }
        }
    }

    /// Build, simulate, and prepare a harvest transaction
    ///
    /// # Arguments
    /// * `farmer_public_key` - The farmer's Stellar public key
    /// * `block_index` - The block index to harvest from
    ///
    /// Returns the transaction XDR (base64) ready for signing
    pub async fn prepare_harvest_transaction(
        &self,
        farmer_public_key: &str,
        block_index: u32,
    ) -> Result<String> {
        // Parse farmer address to ScAddress
        let farmer_strkey = Strkey::from_string(farmer_public_key)?;
        let farmer_address = match farmer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                ScAddress::Account(stellar_xdr::curr::AccountId(
                    stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
                        stellar_xdr::curr::Uint256(pk.0)
                    )
                ))
            }
            _ => anyhow::bail!("Invalid farmer public key type"),
        };

        // Build function arguments for harvest(farmer: Address, index: u32)
        let args = vec![
            ScVal::Address(farmer_address),
            ScVal::U32(block_index),
        ];

        // Build the transaction
        let mut transaction = self.rpc
            .build_invoke_transaction(farmer_public_key, "harvest", args)
            .await?;

        // Simulate to get fees and footprint
        let simulation = self.rpc.simulate_transaction(&transaction).await?;

        // Check for simulation errors
        if let Some(error) = &simulation.error {
            anyhow::bail!("Transaction simulation failed: {}", error);
        }

        // Apply simulation results
        transaction = self.rpc.apply_simulation_to_transaction(transaction, &simulation)?;

        // Wrap the transaction in a TransactionV1Envelope (required for signing)
        let tx_envelope = stellar_xdr::curr::TransactionEnvelope::Tx(
            stellar_xdr::curr::TransactionV1Envelope {
                tx: transaction,
                signatures: stellar_xdr::curr::VecM::default(),
            },
        );

        // Convert the envelope to XDR base64 for signing
        let tx_xdr = tx_envelope.to_xdr_base64(stellar_xdr::curr::Limits::none())?;

        Ok(tx_xdr)
    }

    /// Submit a signed harvest transaction
    ///
    /// # Arguments
    /// * `signed_tx_xdr` - The signed transaction XDR (base64)
    ///
    /// Returns the transaction hash
    pub async fn submit_harvest_transaction(&self, signed_tx_xdr: &str) -> Result<String> {
        self.rpc.submit_transaction(signed_tx_xdr).await
    }

    /// Get the XLM balance of an account
    ///
    /// Returns the balance in stroops, or None if the account doesn't exist
    pub async fn get_xlm_balance(&self, account_address: &str) -> Result<Option<i64>> {
        self.rpc.get_xlm_balance(account_address).await
    }

    /// Check if an account has a KALE trustline
    ///
    /// Returns (has_trustline, balance in stroops)
    pub async fn check_kale_trustline(&self, account_address: &str) -> Result<(bool, i64)> {
        const KALE_ASSET_CODE: &str = "KALE";
        const KALE_ISSUER: &str = "GCHPTWXMT3HYF4RLZHWBNRF4MPXLTJ76ISHMSYIWCCDXWUYOQG5MR2AB";

        self.rpc
            .check_trustline_and_balance(account_address, KALE_ASSET_CODE, KALE_ISSUER)
            .await
    }

    /// Build and prepare a trustline transaction for KALE
    ///
    /// Returns the transaction XDR (base64) ready for signing
    pub async fn prepare_add_kale_trustline_transaction(
        &self,
        account_address: &str,
    ) -> Result<String> {
        const KALE_ASSET_CODE: &str = "KALE";
        const KALE_ISSUER: &str = "GCHPTWXMT3HYF4RLZHWBNRF4MPXLTJ76ISHMSYIWCCDXWUYOQG5MR2AB";

        // Build the trustline transaction
        let transaction = self.rpc
            .build_add_trustline_transaction(account_address, KALE_ASSET_CODE, KALE_ISSUER)
            .await?;

        // Wrap the transaction in a TransactionV1Envelope (required for signing)
        let tx_envelope = stellar_xdr::curr::TransactionEnvelope::Tx(
            stellar_xdr::curr::TransactionV1Envelope {
                tx: transaction,
                signatures: stellar_xdr::curr::VecM::default(),
            },
        );

        // Convert the envelope to XDR base64 for signing
        let tx_xdr = tx_envelope.to_xdr_base64(stellar_xdr::curr::Limits::none())?;

        Ok(tx_xdr)
    }

    /// Submit a signed trustline transaction
    ///
    /// # Arguments
    /// * `signed_tx_xdr` - The signed transaction XDR (base64)
    ///
    /// Returns the transaction hash
    pub async fn submit_trustline_transaction(&self, signed_tx_xdr: &str) -> Result<String> {
        self.rpc.submit_transaction(signed_tx_xdr).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTNET_RPC: &str = "https://soroban-testnet.stellar.org";
    const TESTNET_CONTRACT: &str = "CDSWUUXGPWDZG76ISK6SUCVPZJMD5YUV66J2FXFXFGDX25XKZJIEITAO";
    const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

    #[tokio::test]
    async fn test_get_block_index() -> Result<()> {
        let kale = Kale::new(TESTNET_RPC, TESTNET_CONTRACT, TESTNET_PASSPHRASE)?;
        let block_index = kale.get_block_index().await?;

        println!("Current block index: {}", block_index);

        // Block index should be a reasonable positive number
        assert!(block_index > 0, "Block index should be greater than 0");

        Ok(())
    }
}
