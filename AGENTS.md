# AGENTS.md

This repository is the **StreamingFast fork** of `worldcoin/world-chain`, carrying
Firehose instrumentation on top of upstream World Chain.

## Git remotes

| Remote | Points to | Access |
| --- | --- | --- |
| `origin` | `worldcoin/world-chain` | **read-only** — upstream, we cannot push to it |
| `sf` | `streamingfast/world-chain` | read/write — our fork, this is where our work lands |

The remote naming is a convention, not a guarantee: on some clones the
StreamingFast fork may be named differently (`streamingfast`, `fork`, ...). Resolve
it by URL rather than by name — pick the remote whose URL points at
`streamingfast/world-chain`:

```bash
git remote -v | grep streamingfast
```

**Never push to `origin`.** Pushing our fork's work to the upstream World Chain
repository is wrong even when permissions would allow it.

## Pull requests

Always open PRs **against the StreamingFast fork**, targeting its default branch,
which follows the `release/*` pattern (currently `release/2.x`):

```bash
git push -u sf HEAD
gh pr create --repo streamingfast/world-chain --base release/2.x
```

Do not target `main` — on this fork `main` is a stale mirror of upstream and is
not the integration branch. Do not target a branch on `worldcoin/world-chain`.

The default branch moves as upstream releases advance (`release/2.x` →
`release/3.x` → ...), so confirm it rather than hardcoding:

```bash
gh repo view streamingfast/world-chain --json defaultBranchRef --jq .defaultBranchRef.name
```

Note the local `sf/HEAD` symbolic ref is often stale (it may still point at
`sf/main`) — trust the `gh repo view` output above, not `sf/HEAD`.

Rebase onto the current default branch before pushing, so the PR is conflict-free
and the diff shows only our change.

## Fork-specific files

Our additions are suffixed `.sf` so upstream merges stay clean. Keep changes
confined to these files whenever possible:

- `Dockerfile.sf` — Firehose-enabled image
- `CHANGELOG.sf.md` — our changelog; `.github/workflows/sf-release.yml` reads its
  top section for release notes
- `.github/workflows/sf-release.yml` — our build/push/release pipeline, triggered
  by `v*-fh*` tags and pushes to `release/*` and `firehose/*` branches

Upstream-maintained files (other workflows, crates, etc.) should only be touched
when the Firehose integration genuinely requires it — every edit there is a
future merge conflict.
