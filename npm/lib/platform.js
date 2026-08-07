'use strict';

/**
 * Map Node process.platform/arch → release asset platform id (ADR 0009).
 */
function platformId(p = process.platform, a = process.arch) {
  return p === 'darwin' && a === 'arm64' ? 'darwin-arm64' : null;
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
