import { readFileSync, readdirSync } from 'node:fs'
import { resolve, relative } from 'node:path'
import { expect, it } from 'vitest'

const root = resolve('src')
function files(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(directory, entry.name)
    return entry.isDirectory()
      ? files(path)
      : /\.(ts|vue)$/.test(path)
        ? [path]
        : []
  })
}

it('keeps business code independent of legacy APIs and types', () => {
  for (const path of files(root)) {
    const name = relative(root, path).replaceAll('\\', '/')
    if (name.startsWith('api/legacy/') || name === 'types/legacy.ts') continue
    const text = readFileSync(path, 'utf8')
    expect(text, name).not.toMatch(
      /(?:from\s*|import\s*\()['"][^'"]*(?:api\/compat|api\/legacy|types\/legacy)/
    )
    if (name !== 'api/platformExceptions.ts')
      expect(text, name).not.toMatch(/['"]legacy_[a-z_]+['"]/)
    if (!name.startsWith('api/'))
      expect(text, name).not.toMatch(/from ['"]@tauri-apps\/api\/core['"]/)
  }
})

it('limits the approved exceptions to the existing three commands', () => {
  const text = readFileSync(resolve(root, 'api/platformExceptions.ts'), 'utf8')
  expect(
    [...text.matchAll(/['"](legacy_[a-z_]+)['"]/g)]
      .map((match) => match[1])
      .sort()
  ).toEqual([
    'legacy_check_orphan_records',
    'legacy_opened_urls',
    'legacy_purge_synced_deletions'
  ])
})
