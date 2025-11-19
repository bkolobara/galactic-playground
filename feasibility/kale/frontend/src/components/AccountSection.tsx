import React from "react";
import { AuthState } from "../types";
import { styles } from "../styles";

interface AccountSectionProps {
  authState: AuthState;
  onConnect: () => void;
  onLogout: () => void;
}

export const AccountSection: React.FC<AccountSectionProps> = ({
  authState,
  onConnect,
  onLogout,
}) => {
  return (
    <>
      <div style={styles.accountRow}>
        {authState.status === "success" ? (
          <>
            <div style={styles.accountInfo}>
              <span style={styles.label}>Account:</span>
              <a
                href={`https://stellar.expert/explorer/testnet/account/${authState.publicKey}`}
                target="_blank"
                style={styles.value}
              >
                {authState.publicKey}
              </a>
            </div>
            <button style={styles.logoutButton} onClick={onLogout}>
              Logout
            </button>
          </>
        ) : authState.status === "loading" ? (
          <>
            <div style={styles.accountInfo}>
              <span style={styles.label}>Account:</span>
              <span style={styles.hint}>Connecting to Albedo...</span>
            </div>
          </>
        ) : (
          <button style={styles.connectButton} onClick={onConnect}>
            Connect Wallet
          </button>
        )}
      </div>
      {authState.status === "error" && (
        <div style={styles.errorBanner}>
          Authentication error: {authState.error}
        </div>
      )}
    </>
  );
};
