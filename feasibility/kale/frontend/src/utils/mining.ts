import { keccak256 } from 'js-sha3';
import { StrKey } from '@stellar/stellar-base';

/**
 * Count leading zeros in a hash (hex representation)
 */
export const countLeadingZeros = (hash: Uint8Array): number => {
  let zeros = 0;
  for (const byte of hash) {
    if (byte === 0) {
      zeros += 2; // Two hex digits
    } else {
      // Count leading zeros in the nibble
      zeros += byte < 16 ? 1 : 0;
      break;
    }
  }
  return zeros;
};

/**
 * Calculate Keccak256 hash for KALE farming
 * Hash = Keccak256(block_index || nonce || entropy || farmer_address)
 */
export const calculateHash = (
  blockIndex: number,
  nonce: number,
  entropy: string,
  farmerAddress: string
): Uint8Array => {
  // Build the 76-byte input: block_index (4) + nonce (8) + entropy (32) + farmer_address (32)
  const buffer = new Uint8Array(76);

  // Block index (4 bytes, big-endian u32)
  const indexView = new DataView(buffer.buffer, 0, 4);
  indexView.setUint32(0, blockIndex, false); // false = big-endian

  // Nonce (8 bytes, big-endian u64)
  const nonceView = new DataView(buffer.buffer, 4, 8);
  // JavaScript numbers are 53-bit safe, so we can use setUint32 for both halves
  nonceView.setUint32(0, Math.floor(nonce / 0x100000000), false); // high 32 bits
  nonceView.setUint32(4, nonce & 0xFFFFFFFF, false); // low 32 bits

  // Entropy (32 bytes)
  const entropyBytes = new Uint8Array(
    entropy.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
  );
  buffer.set(entropyBytes, 12);

  // Farmer address (32 bytes) - decode the Stellar G... address to raw bytes
  const addressBytes = StrKey.decodeEd25519PublicKey(farmerAddress);
  buffer.set(addressBytes, 44);

  // Calculate Keccak256
  const hashHex = keccak256(buffer);
  return new Uint8Array(
    hashHex.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
  );
};
