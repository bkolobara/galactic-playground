import React from "react";
import { AccountStatus, PailData, PlantState } from "../types";
import { styles } from "../styles";
import { MIN_XLM_BALANCE } from "../constants";

interface PlantSectionProps {
  accountStatus: AccountStatus | null;
  currentPailData: PailData | null;
  publicKey?: string;
  plantState: PlantState;
  onPlant: () => void;
}

export const PlantSection: React.FC<PlantSectionProps> = ({
  accountStatus,
  currentPailData,
  publicKey,
  plantState,
  onPlant,
}) => {
  const isReadyToPlant = () => {
    if (!accountStatus) return false;
    if (!accountStatus.exists) return false;
    if (accountStatus.xlmBalance < MIN_XLM_BALANCE) return false;
    if (!accountStatus.hasTrustline) return false;
    return true;
  };

  const isPlantProcessing =
    plantState.status === "preparing" ||
    plantState.status === "signing" ||
    plantState.status === "submitting";

  return (
    <div style={styles.actionSection}>
      {currentPailData?.hasPail ? (
        <div style={styles.statusMessage}>✓ Kale planted in current field!</div>
      ) : null}
      <button
        style={{
          ...styles.button,
          ...(currentPailData?.hasPail || !isReadyToPlant() || isPlantProcessing
            ? styles.buttonDisabled
            : {}),
        }}
        onClick={onPlant}
        disabled={
          currentPailData?.hasPail || !isReadyToPlant() || isPlantProcessing
        }
      >
        {isPlantProcessing ? "Planting..." : "Plant KALE seed!"}
      </button>

      {/* Loading messages */}
      {isPlantProcessing && (
        <div
          style={{
            fontSize: "12px",
            color: "#666",
            fontStyle: "italic",
            marginTop: "4px",
          }}
        >
          {plantState.status === "preparing" &&
            "Preparing plant transaction..."}
          {plantState.status === "signing" && "Waiting for signature..."}
          {plantState.status === "submitting" &&
            "Submitting plant transaction..."}
        </div>
      )}

      {/* Error messages */}
      {plantState.status === "error" && (
        <div style={styles.errorBanner}>{plantState.error}</div>
      )}

      {/* Hint messages */}
      {!isReadyToPlant() && !currentPailData?.hasPail && !isPlantProcessing && (
        <p style={styles.hint}>
          {!publicKey
            ? "Connect wallet to plant"
            : !accountStatus?.exists ||
              accountStatus.xlmBalance < MIN_XLM_BALANCE
            ? "Fund account first"
            : !accountStatus?.hasTrustline
            ? "Add KALE trustline first"
            : "Checking requirements..."}
        </p>
      )}
    </div>
  );
};
