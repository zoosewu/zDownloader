// Write the updater's latest.json from the platforms already uploaded to a release, and attach it.
// Each build job runs this after uploading its own bundle, so a platform is offered as soon as it
// is built; the release workflow's last job runs it again with --all, which rewrites the file from
// every bundle (fixing an entry lost when two jobs wrote at once) and fails if a platform is
// missing. Each platform's entry is the bundle the updater installs plus the contents of its .sig
// file (created with TAURI_SIGNING_PRIVATE_KEY).
//   node scripts/updater-json.mjs [--all]
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
const requireAll = process.argv.includes('--all');

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

/** latest.json for the bundles uploaded so far, and the platforms that have none yet. */
async function latest() {
  const release = await api(`/repos/${repo}/releases/${releaseId}`);
  const assets = await api(`/repos/${repo}/releases/${releaseId}/assets?per_page=100`);
  const platforms = {};
  const missing = [];
  for (const [platform, suffix] of Object.entries(PLATFORMS)) {
    const bundle = assets.find((a) => a.name.endsWith(suffix));
    const sig = bundle && assets.find((a) => a.name === `${bundle.name}.sig`);
    if (!bundle || !sig) {
      missing.push(platform);
      continue;
    }
    platforms[platform] = {
      signature: (await api(`/repos/${repo}/releases/assets/${sig.id}`, 'application/octet-stream')).trim(),
      // The published URL (a draft's own download URL changes when it is published).
      url: `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(bundle.name)}`,
    };
  }
  const body = JSON.stringify(
    { version: tag.replace(/^v/, ''), notes: release.body ?? '', pub_date: new Date().toISOString(), platforms },
    null,
    2,
  );
  return { body, missing, old: assets.find((a) => a.name === 'latest.json') };
}

/** Replace latest.json; false when another job uploaded one in between (then read again). */
async function replace({ body, old }) {
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
  if (upload.status === 422) return false; // already_exists
  if (!upload.ok) throw new Error(`upload latest.json: ${upload.status} ${await upload.text()}`);
  return true;
}

for (let attempt = 1; ; attempt++) {
  const result = await latest();
  if (result.missing.length === Object.keys(PLATFORMS).length) throw new Error('no platform is uploaded yet');
  if (await replace(result)) {
    console.log(result.body);
    if (requireAll && result.missing.length) {
      throw new Error(`no bundle with a .sig for ${result.missing.join(', ')}: re-run the failed jobs`);
    }
    break;
  }
  if (attempt === 5) throw new Error('latest.json kept changing while it was written');
  await new Promise((r) => setTimeout(r, 2000 + Math.random() * 3000));
}
