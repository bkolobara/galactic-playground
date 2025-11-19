import React from "react";
import { useAuth } from "./hooks/useAuth";
import { useUserData } from "./hooks/useUserData";
import { useTransactions } from "./hooks/useTransactions";
import { Header } from "./components/Header";
import { ContractInfo } from "./components/ContractInfo";
import { AccountSection } from "./components/AccountSection";
import { FundingSection } from "./components/FundingSection";
import { TrustlineSection } from "./components/TrustlineSection";
import { PlantSection } from "./components/PlantSection";
import { FieldCard } from "./components/FieldCard";
import { TransactionProgress } from "./components/TransactionProgress";
import { styles } from "./styles";

const App: React.FC = () => {
  // User data hook
  const {
    accountStatus,
    blockInfo,
    currentPailData,
    fields,
    loadUserData,
    resetUserData,
    setAccountStatus,
    setBlockInfo,
    setCurrentPailData,
    setHarvestableBlocks,
    isTransacting,
    setIsTransacting,
  } = useUserData();

  // Auth hook
  const { authState, handleConnectWallet, handleLogout } =
    useAuth(loadUserData);

  // Transactions hook
  const {
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
  } = useTransactions({
    publicKey: authState.publicKey,
    blockInfo,
    currentPailData,
    accountStatus,
    setBlockInfo,
    setCurrentPailData,
    setAccountStatus,
    loadUserData,
    setIsTransacting,
    onHarvestSuccess: (blockIndex) => {
      // Remove the harvested block from the list
      setHarvestableBlocks((prev) => prev.filter((b) => b !== blockIndex));
    },
  });

  // Logout handler with state reset
  const handleLogoutWithReset = () => {
    handleLogout();
    resetUserData();
  };

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <Header />

        {/* Main UI - Unified View */}
        {transactionStep === "idle" && (
          <div style={styles.unifiedContainer}>
            <ContractInfo />

            <AccountSection
              authState={authState}
              onConnect={handleConnectWallet}
              onLogout={handleLogoutWithReset}
            />

            {/* Funding and Trustline Sections */}
            {authState.status === "success" && accountStatus && (
              <>
                <FundingSection
                  accountStatus={accountStatus}
                  fundingState={fundingState}
                  onFund={handleFundAccount}
                />

                <TrustlineSection
                  accountStatus={accountStatus}
                  trustlineState={trustlineState}
                  onAddTrustline={handleAddTrustline}
                />
              </>
            )}

            {/* Plant Section - above field cards */}
            {authState.status === "success" && accountStatus && blockInfo && (
              <PlantSection
                accountStatus={accountStatus}
                currentPailData={currentPailData}
                publicKey={authState.publicKey}
                plantState={plantState}
                onPlant={handlePlantTransaction}
              />
            )}

            {/* Field Cards */}
            {fields.map((field) => (
              <FieldCard
                key={field.blockIndex}
                field={field}
                accountStatus={accountStatus}
                publicKey={authState.publicKey}
                workState={workState}
                harvestState={harvestState}
                onWork={handleWorkTransaction}
                onHarvest={handleHarvestTransaction}
              />
            ))}

            {/* Remaining fields message */}
            {blockInfo && blockInfo.blockIndex >= 5 && (
              <p style={{
                ...styles.hint,
                textAlign: 'center',
                marginTop: '20px',
                fontSize: '12px',
                color: '#999'
              }}>
                There are {blockInfo.blockIndex - 4} more fields...
              </p>
            )}
          </div>
        )}

        {/* Transaction Progress and Success Screens */}
        <TransactionProgress
          transactionStep={transactionStep}
          plantState={plantState}
          workState={workState}
          harvestState={harvestState}
          trustlineState={trustlineState}
          blockInfo={blockInfo}
          onPlantRetry={handlePlantTransaction}
          onWorkRetry={handleWorkTransaction}
          onBackToFarm={() => setTransactionStep("idle")}
        />
      </div>
    </div>
  );
};

export default App;
