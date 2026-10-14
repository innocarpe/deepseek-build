'use strict';

/**
 * Map Node process.platform/arch → release asset platform id (ADR 0009, as
 * amended 2026-10-15: darwin-arm64 and linux-x64 are the released targets).
 * Every other pair returns null so the caller fails with a clear message.
 */
function platformId(p = process.platform, a = process.arch) {
  if (p === 'darwin' && a === 'arm64') return 'darwin-arm64';
  if (p === 'linux' && a === 'x64') return 'linux-x64';
  return null;
}

function releaseAssetName(version, platform) {
  return `deepseek-build-${version}-${platform}.tar.gz`;
}

function releaseDownloadUrl(version, platform) {
  const name = releaseAssetName(version, platform);
  return `https://github.com/innocarpe/deepseek-build/releases/download/v${version}/${name}`;
}

module.exports = {
  platformId,
  releaseAssetName,
  releaseDownloadUrl,
};
