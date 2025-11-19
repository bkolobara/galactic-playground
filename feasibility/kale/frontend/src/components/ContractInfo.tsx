import React from "react";
import { styles } from "../styles";
import { CONTRACT_ADDRESS } from "../constants";

export const ContractInfo: React.FC = () => {
  return (
    <div style={styles.contractRow}>
      <span style={styles.contractLabel}>Contract:</span>
      <a
        href={`https://stellar.expert/explorer/testnet/contract/${CONTRACT_ADDRESS}`}
        target="_blank"
        style={styles.contractHash}
      >
        {CONTRACT_ADDRESS}
      </a>
    </div>
  );
};
