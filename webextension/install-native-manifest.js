#!/usr/bin/env node

const fs = require('fs');
const os = require('os');
const path = require('path');

function main() {
  let bridgePath = path.join(__dirname, '..', 'toodlext', 'target',
    'debug', 'toodlext');
  let fullBridgePath = fs.realpathSync(bridgePath);

  let nativeManifest = require('./native-manifest.json');
  nativeManifest.path = fullBridgePath;

  let manifestsPath = path.join(os.homedir(), 'Library', 'Application Support',
    'Mozilla', 'NativeMessagingHosts');
  fs.writeFileSync(path.join(manifestsPath, 'toodlext.json'),
    JSON.stringify(nativeManifest));
}

main();
