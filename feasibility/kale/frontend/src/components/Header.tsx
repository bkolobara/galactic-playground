import React from 'react';
import { styles } from '../styles';

export const Header: React.FC = () => {
  return (
    <div style={styles.titleRow}>
      <h1 style={styles.title}>KALE Farming</h1>
      <div style={styles.testnetBadge}>testnet</div>
    </div>
  );
};
