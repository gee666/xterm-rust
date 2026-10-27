import { statfs } from 'node:fs/promises';
const MIB = 1024 * 1024;
export const DEFAULT_MIN_FREE_MIB = 1024;
export async function diskHeadroom(root, minFreeMiB) {
  const value = await statfs(root, { bigint: true });
  const available = value.bavail * value.bsize;
  const minimum = BigInt(minFreeMiB) * BigInt(MIB);
  return { availableBytes: available.toString(), minimumBytes: minimum.toString(), sufficient: available >= minimum };
}
export function requireHeadroom(observation) {
  if (!observation.sufficient) {
    throw new Error(`Insufficient disk headroom: ${observation.availableBytes} bytes available, ${observation.minimumBytes} required. No cleanup was attempted.`);
  }
}
