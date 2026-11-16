// Write the updater's latest.json for a release whose builds are all uploaded, and attach it.
// One job does this after every platform finished, so parallel builds cannot overwrite each
// other's entries. Each platform's entry is the bundle the updater installs plus the contents
// of its .sig file (created with TAURI_SIGNING_PRIVATE_KEY).
//   env: GITHUB_TOKEN, GITHUB_REPOSITORY (owner/repo), RELEASE_ID, TAG (vX.Y.Z)

// Failures become workflow annotations, which the public API shows (job logs need a login).
process.on('uncaughtException', (e) => {
  console.log(`::error title=latest.json::${String(e.message).replace(/\r?\n/g, '%0A')}`);
  process.exit(1);
});

const { GITHUB_TOKEN, GITHUB_REPOSITORY: repo, RELEASE_ID: releaseId, TAG: tag } = process.env;
for (const [name, value] of Object.entries({ GITHUB_TOKEN, GITHUB_REPOSITORY: repo, RELEASE_ID: releaseId, TAG: tag })) {
  if (!value) throw new Error(`${name} is not set`);
}

/** Updater platform key -> how its signed bundle's name ends (as tauri-action uploads it). */
const PLATFORMS = {
  'windows-x86_64': '_x64-setup.exe',
  'darwin-aarch64': '_aarch64.app.tar.gz',
  'darwin-x86_64': '_x64.app.tar.gz',
  'linux-x86_64': '_amd64.AppImage',
};

const api = async (path, accept = 'application/vnd.github+json') => {
  const res = await fetch(`https://api.github.com${path}`, {
    headers: { authorization: `Bearer ${GITHUB_TOKEN}`, accept, 'x-github-api-version': '2022-11-28' },
  });
  if (!res.ok) throw new Error(`GET ${path}: ${res.status} ${await res.text()}`);
  return accept.includes('json') ? res.json() : res.text();
};

const release =await api(`/repos/${repo}/releases/${releaseId}`);
const assets = await api(`/repos/${repo}/releases/${releaseId}/assets?per_page=100`);

const platforms = {};
for (const [platform, suffix] of Object.entries(PLATFORMS)) {
  const bundle = assets.find((a) => a.name.endsWith(suffix));
  const sig = bundle && assets.find((a) => a.name === `${bundle.name}.sig`);
  if (!bundle || !sig) {
    throw new Error(`${platform}: no *${suffix} with a .sig. Assets: ${assets.map((a) => a.name).join(', ')}`);
  }
  platforms[platform] = {
    signature: (await api(`/repos/${repo}/releases/assets/${sig.id}`, 'application/octet-stream')).trim(),
    // The published URL (a draft's own download URL changes when it is published).
    url: `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(bundle.name)}`,
  };
}

const latest = {
  version: tag.replace(/^v/, ''),
  notes: release.body ?? '',
  pub_date: new Date().toISOString(),
  platforms,
};
const body = JSON.stringify(latest, null, 2);

const old = assets.find((a) => a.name === 'latest.json');
if (old) {
  await fetch(`https://api.github.com/repos/${repo}/releases/assets/${old.id}`, {
    method: 'DELETE',
    headers: { authorization: `Bearer ${GITHUB_TOKEN}` },
  });
}
const upload = await fetch(`https://uploads.github.com/repos/${repo}/releases/${releaseId}/assets?name=latest.json`, {
  method: 'POST',
  headers: { authorization: `Bearer ${GITHUB_TOKEN}`, 'content-type': 'application/json' },
  body,
});
if (!upload.ok) throw new Error(`upload latest.json: ${upload.status} ${await upload.text()}`);
console.log(body);
