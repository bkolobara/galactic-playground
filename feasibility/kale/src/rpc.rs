use anyhow::{Context, Result};
use stellar_rpc_client::{Client, LedgerEntryResult, SimulateTransactionResponse};
use stellar_strkey::{Contract, Strkey};
use stellar_xdr::curr::{
    Hash, Limits, MuxedAccount, Operation, OperationBody, Preconditions, PublicKey, ReadXdr,
    ScAddress, ScVal, SequenceNumber, Transaction, TransactionEnvelope, TransactionExt, Uint256,
};

/// General-purpose Soroban RPC client for interacting with contracts
pub struct SorobanRpc {
    client: Client,
    contract_id: Contract,
    network_passphrase: String,
}

impl SorobanRpc {
    /// Create a new RPC client instance
    ///
    /// # Arguments
    /// * `rpc_url` - The Soroban RPC endpoint URL
    /// * `contract_address` - The contract address (e.g., "CDSWUUXGPWDZG76ISK6SUCVPZJMD5YUV66J2FXFXFGDX25XKZJIEITAO")
    /// * `network_passphrase` - The network passphrase (e.g., "Test SDF Network ; September 2015" for testnet)
    pub fn new(rpc_url: &str, contract_address: &str, network_passphrase: &str) -> Result<Self> {
        let client = Client::new(rpc_url)?;
        let contract_id =
            Contract::from_string(contract_address).context("Failed to parse contract address")?;

        Ok(Self {
            client,
            contract_id,
            network_passphrase: network_passphrase.to_string(),
        })
    }

    /// Get the contract instance storage entry
    ///
    /// Instance storage is accessed using ScVal::LedgerKeyContractInstance as the key.
    pub async fn get_contract_instance(&self) -> Result<LedgerEntryResult> {
        let contract_address = ScAddress::Contract(Hash(self.contract_id.0.clone()));

        // Construct the ledger key for contract instance storage
        let ledger_key =
            stellar_xdr::curr::LedgerKey::ContractData(stellar_xdr::curr::LedgerKeyContractData {
                contract: contract_address,
                key: stellar_xdr::curr::ScVal::LedgerKeyContractInstance,
                durability: stellar_xdr::curr::ContractDataDurability::Persistent,
            });

        // Fetch the contract instance entry
        let response = self.client.get_ledger_entries(&[ledger_key]).await?;

        response
            .entries
            .and_then(|e| e.into_iter().next())
            .context("Contract instance entry not found")
    }

    /// Parse a value from instance storage by key name
    pub fn parse_instance_storage_value(
        entry: &LedgerEntryResult,
        key_name: &str,
    ) -> Result<ScVal> {
        // Decode the LedgerEntryData from base64 XDR
        let entry_data =
            stellar_xdr::curr::LedgerEntryData::from_xdr_base64(&entry.xdr, Limits::none())
                .context("Failed to decode XDR")?;

        // Navigate: ContractData -> ContractInstance -> storage -> find key
        if let stellar_xdr::curr::LedgerEntryData::ContractData(contract_data) = entry_data {
            if let ScVal::ContractInstance(instance) = contract_data.val {
                let storage_map = instance
                    .storage
                    .context("ContractInstance has no storage map")?;

                // Instance storage keys are wrapped: Vec(ScVec([Symbol("KeyName")]))
                for map_entry in storage_map.iter() {
                    if let ScVal::Vec(Some(vec)) = &map_entry.key {
                        if let Some(ScVal::Symbol(sym)) = vec.first() {
                            if sym.to_utf8_string_lossy() == key_name {
                                return Ok(map_entry.val.clone());
                            }
                        }
                    }
                }

                anyhow::bail!("Key '{}' not found in instance storage", key_name);
            }
        }

        anyhow::bail!("Unexpected ledger entry structure")
    }

    /// Build a transaction to invoke a contract function
    ///
    /// # Arguments
    /// * `source_account` - The public key of the account that will sign the transaction
    /// * `function_name` - The contract function to invoke
    /// * `args` - The function arguments
    pub async fn build_invoke_transaction(
        &self,
        source_account: &str,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<Transaction> {
        // Parse the source account public key
        let source_strkey =
            Strkey::from_string(source_account).context("Failed to parse source account")?;

        let source_public_key = match source_strkey {
            Strkey::PublicKeyEd25519(pk) => PublicKey::PublicKeyTypeEd25519(Uint256(pk.0)),
            _ => anyhow::bail!("Invalid source account key type"),
        };

        // Get the account sequence number
        let account_response = self.client.get_account(source_account).await?;
        let sequence = account_response.seq_num.0 as i64 + 1;

        // Extract bytes from public key
        let account_bytes = match source_public_key {
            PublicKey::PublicKeyTypeEd25519(ref uint256) => uint256.0,
        };

        // Build the invoke contract host function
        let contract_address = ScAddress::Contract(Hash(self.contract_id.0.clone()));
        let function_symbol = stellar_xdr::curr::ScSymbol(
            function_name.try_into().context("Function name too long")?,
        );

        let invoke_args = stellar_xdr::curr::InvokeContractArgs {
            contract_address,
            function_name: function_symbol,
            args: args.try_into()?,
        };

        let host_function = stellar_xdr::curr::HostFunction::InvokeContract(invoke_args);

        // Create the invoke host function operation
        let operation = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(stellar_xdr::curr::InvokeHostFunctionOp {
                host_function,
                auth: stellar_xdr::curr::VecM::default(),
            }),
        };

        // Build the transaction (fees will be updated after simulation)
        let transaction = Transaction {
            source_account: MuxedAccount::Ed25519(Uint256(account_bytes)),
            fee: 100, // Placeholder, will be updated after simulation
            seq_num: SequenceNumber(sequence),
            cond: Preconditions::None,
            memo: stellar_xdr::curr::Memo::None,
            operations: vec![operation].try_into()?,
            ext: TransactionExt::V0,
        };

        Ok(transaction)
    }

    /// Simulate a transaction to get resource requirements and fees
    pub async fn simulate_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<SimulateTransactionResponse> {
        // Wrap transaction in envelope for simulation
        let envelope = TransactionEnvelope::Tx(stellar_xdr::curr::TransactionV1Envelope {
            tx: transaction.clone(),
            signatures: stellar_xdr::curr::VecM::default(),
        });

        self.client
            .simulate_transaction_envelope(&envelope)
            .await
            .context("Failed to simulate transaction")
    }

    /// Apply simulation results to a transaction
    pub fn apply_simulation_to_transaction(
        &self,
        mut transaction: Transaction,
        simulation: &SimulateTransactionResponse,
    ) -> Result<Transaction> {
        // Extract simulation results
        let first_result = simulation
            .results
            .first()
            .context("No simulation results found")?;

        // Get the transaction data from the simulation response
        // Check if transaction_data is empty (simulation might not need Soroban data)
        if simulation.transaction_data.is_empty() {
            anyhow::bail!("No transaction data in simulation response");
        }

        // Parse the SorobanTransactionData
        let soroban_tx_data = stellar_xdr::curr::SorobanTransactionData::from_xdr_base64(
            &simulation.transaction_data,
            Limits::none(),
        )
        .context("Failed to parse soroban transaction data")?;

        // Update the transaction with Soroban data
        // Extract auth from simulation if available
        if !first_result.auth.is_empty() {
            // Parse auth entries
            let auth_entries: Vec<stellar_xdr::curr::SorobanAuthorizationEntry> = first_result
                .auth
                .iter()
                .filter_map(|xdr| {
                    stellar_xdr::curr::SorobanAuthorizationEntry::from_xdr_base64(
                        xdr,
                        Limits::none(),
                    )
                    .ok()
                })
                .collect();

            // Convert VecM to Vec, modify, and convert back
            let mut operations: Vec<_> = transaction.operations.to_vec();
            if let Some(operation) = operations.get_mut(0) {
                if let OperationBody::InvokeHostFunction(ref mut invoke_op) = operation.body {
                    invoke_op.auth = auth_entries
                        .try_into()
                        .context("Failed to convert auth entries")?;
                }
            }
            transaction.operations = operations.try_into()?;
        }

        // Update transaction extension with Soroban data
        transaction.ext = TransactionExt::V1(stellar_xdr::curr::SorobanTransactionData {
            ext: stellar_xdr::curr::ExtensionPoint::V0,
            resources: soroban_tx_data.resources,
            resource_fee: soroban_tx_data.resource_fee,
        });

        // Update fee with simulation results
        let resource_fee = simulation.min_resource_fee as i64;
        let base_fee = 100i64; // Base inclusion fee
        transaction.fee = (base_fee + resource_fee) as u32;

        Ok(transaction)
    }

    /// Submit a signed transaction to the network
    pub async fn submit_transaction(&self, signed_tx_xdr: &str) -> Result<String> {
        let envelope = TransactionEnvelope::from_xdr_base64(signed_tx_xdr, Limits::none())
            .context("Failed to parse signed transaction XDR")?;

        let response = self
            .client
            .send_transaction(&envelope)
            .await
            .context("Failed to submit transaction")?;

        // Convert Hash to hex string
        let hash_hex = hex::encode(response.0);

        Ok(hash_hex)
    }

    /// Check if an account has a trustline to a specific asset and get the balance
    ///
    /// # Arguments
    /// * `account_address` - The account's public key
    /// * `asset_code` - The asset code (e.g., "KALE")
    /// * `asset_issuer` - The asset issuer's public key
    ///
    /// Returns (has_trustline, balance) where balance is in stroops
    pub async fn check_trustline_and_balance(
        &self,
        account_address: &str,
        asset_code: &str,
        asset_issuer: &str,
    ) -> Result<(bool, i64)> {
        // Parse addresses
        let account_strkey =
            Strkey::from_string(account_address).context("Failed to parse account address")?;
        let issuer_strkey =
            Strkey::from_string(asset_issuer).context("Failed to parse issuer address")?;

        let account_id = match account_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                stellar_xdr::curr::AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(pk.0)))
            }
            _ => anyhow::bail!("Invalid account key type"),
        };

        let issuer_id = match issuer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                stellar_xdr::curr::AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(pk.0)))
            }
            _ => anyhow::bail!("Invalid issuer key type"),
        };

        // Create TrustLineAsset
        let asset = if asset_code.len() <= 4 {
            stellar_xdr::curr::TrustLineAsset::CreditAlphanum4(stellar_xdr::curr::AlphaNum4 {
                asset_code: stellar_xdr::curr::AssetCode4(
                    asset_code
                        .as_bytes()
                        .iter()
                        .chain(std::iter::repeat(&0u8))
                        .take(4)
                        .copied()
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                ),
                issuer: issuer_id.clone(),
            })
        } else {
            stellar_xdr::curr::TrustLineAsset::CreditAlphanum12(stellar_xdr::curr::AlphaNum12 {
                asset_code: stellar_xdr::curr::AssetCode12(
                    asset_code
                        .as_bytes()
                        .iter()
                        .chain(std::iter::repeat(&0u8))
                        .take(12)
                        .copied()
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                ),
                issuer: issuer_id,
            })
        };

        // Construct the ledger key for the trustline
        let trustline_key =
            stellar_xdr::curr::LedgerKey::Trustline(stellar_xdr::curr::LedgerKeyTrustLine {
                account_id: account_id.clone(),
                asset,
            });

        // Try to get the trustline ledger entry
        match self.client.get_ledger_entries(&[trustline_key]).await {
            Ok(response) => {
                if let Some(entries) = response.entries {
                    if let Some(entry) = entries.first() {
                        // Parse the trustline entry to get the balance
                        let entry_data = stellar_xdr::curr::LedgerEntryData::from_xdr_base64(
                            &entry.xdr,
                            Limits::none(),
                        )?;

                        if let stellar_xdr::curr::LedgerEntryData::Trustline(trustline) = entry_data
                        {
                            // Return (true, balance)
                            return Ok((true, trustline.balance));
                        }
                    }
                }
                // Trustline doesn't exist
                Ok((false, 0))
            }
            Err(_) => {
                // If the request failed or entry doesn't exist, trustline doesn't exist
                Ok((false, 0))
            }
        }
    }

    /// Get the network passphrase
    pub fn network_passphrase(&self) -> &str {
        &self.network_passphrase
    }

    /// Get a ledger entry by key (exposed for custom queries)
    pub async fn get_ledger_entry(
        &self,
        key: stellar_xdr::curr::LedgerKey,
    ) -> Result<Option<LedgerEntryResult>> {
        match self.client.get_ledger_entries(&[key]).await {
            Ok(response) => Ok(response.entries.and_then(|e| e.into_iter().next())),
            Err(_) => Ok(None),
        }
    }

    /// Get the contract ID (exposed for building custom ledger keys)
    pub fn contract_id(&self) -> &Contract {
        &self.contract_id
    }

    /// Get the XLM balance of an account
    ///
    /// # Arguments
    /// * `account_address` - The account's public key
    ///
    /// Returns the balance in stroops, or None if the account doesn't exist
    pub async fn get_xlm_balance(&self, account_address: &str) -> Result<Option<i64>> {
        match self.client.get_account(account_address).await {
            Ok(account) => {
                // Balance is already an i64 in the account response
                Ok(Some(account.balance))
            }
            Err(_) => {
                // Account doesn't exist
                Ok(None)
            }
        }
    }

    /// Build a transaction to add a trustline
    ///
    /// # Arguments
    /// * `source_account` - The public key of the account adding the trustline
    /// * `asset_code` - The asset code (e.g., "KALE")
    /// * `asset_issuer` - The asset issuer's public key
    pub async fn build_add_trustline_transaction(
        &self,
        source_account: &str,
        asset_code: &str,
        asset_issuer: &str,
    ) -> Result<Transaction> {
        // Parse the source account public key
        let source_strkey =
            Strkey::from_string(source_account).context("Failed to parse source account")?;

        let source_public_key = match source_strkey {
            Strkey::PublicKeyEd25519(pk) => PublicKey::PublicKeyTypeEd25519(Uint256(pk.0)),
            _ => anyhow::bail!("Invalid source account key type"),
        };

        // Parse issuer
        let issuer_strkey =
            Strkey::from_string(asset_issuer).context("Failed to parse issuer address")?;

        let issuer_id = match issuer_strkey {
            Strkey::PublicKeyEd25519(pk) => {
                stellar_xdr::curr::AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(pk.0)))
            }
            _ => anyhow::bail!("Invalid issuer key type"),
        };

        // Get the account sequence number
        let account_response = self.client.get_account(source_account).await?;
        let sequence = account_response.seq_num.0 as i64 + 1;

        // Extract bytes from public key
        let account_bytes = match source_public_key {
            PublicKey::PublicKeyTypeEd25519(ref uint256) => uint256.0,
        };

        // Create the asset (ChangeTrustAsset type for ChangeTrust operation)
        let asset = if asset_code.len() <= 4 {
            stellar_xdr::curr::ChangeTrustAsset::CreditAlphanum4(stellar_xdr::curr::AlphaNum4 {
                asset_code: stellar_xdr::curr::AssetCode4(
                    asset_code
                        .as_bytes()
                        .iter()
                        .chain(std::iter::repeat(&0u8))
                        .take(4)
                        .copied()
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                ),
                issuer: issuer_id,
            })
        } else {
            stellar_xdr::curr::ChangeTrustAsset::CreditAlphanum12(stellar_xdr::curr::AlphaNum12 {
                asset_code: stellar_xdr::curr::AssetCode12(
                    asset_code
                        .as_bytes()
                        .iter()
                        .chain(std::iter::repeat(&0u8))
                        .take(12)
                        .copied()
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                ),
                issuer: issuer_id,
            })
        };

        // Create ChangeTrust operation
        let operation = Operation {
            source_account: None,
            body: OperationBody::ChangeTrust(stellar_xdr::curr::ChangeTrustOp {
                line: asset,
                limit: i64::MAX, // Maximum limit
            }),
        };

        // Build the transaction
        let transaction = Transaction {
            source_account: MuxedAccount::Ed25519(Uint256(account_bytes)),
            fee: 100, // Base fee for simple operations
            seq_num: SequenceNumber(sequence),
            cond: Preconditions::None,
            memo: stellar_xdr::curr::Memo::None,
            operations: vec![operation].try_into()?,
            ext: TransactionExt::V0,
        };

        Ok(transaction)
    }
}
