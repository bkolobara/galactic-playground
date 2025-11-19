import React from 'react';
import { styles } from '../styles';

interface HarvestSectionProps {
  harvestableBlocks: number[];
  onHarvest: (blockIndex: number) => void;
}

export const HarvestSection: React.FC<HarvestSectionProps> = ({
  harvestableBlocks,
  onHarvest,
}) => {
  return (
    <div style={styles.actionSection}>
      <h3 style={styles.sectionTitle}>Harvest</h3>
      {harvestableBlocks.length > 0 ? (
        <>
          <div style={styles.statusMessage}>
            {harvestableBlocks.length} block(s) ready to harvest
          </div>
          {harvestableBlocks.map((blockIndex) => (
            <button
              key={blockIndex}
              style={{...styles.button, ...styles.harvestButton}}
              onClick={() => onHarvest(blockIndex)}
            >
              Harvest field {blockIndex}
            </button>
          ))}
        </>
      ) : (
        <p style={styles.hint}>No fields ready for harvest yet</p>
      )}
    </div>
  );
};
