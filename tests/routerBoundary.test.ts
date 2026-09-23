import { readdirSync, readFileSync } from 'node:fs'
import { relative, resolve } from 'node:path'

import ts from 'typescript'
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
function parse(text: string) {
  return ts.createSourceFile('source.ts', text, ts.ScriptTarget.Latest, true)
}

// Derive the exception list from registered lazy components, not all of views/.
const catalog = parse(readFileSync(resolve(root, 'router/catalog.ts'), 'utf8'))
const destinations = new Map<string, string>()
function collect(node: ts.Node) {
  if (ts.isObjectLiteralExpression(node)) {
    const properties = node.properties.filter(ts.isPropertyAssignment)
    const name = properties.find(
      (property) => property.name.getText(catalog) === 'name'
    )
    const component = properties.find(
      (property) => property.name.getText(catalog) === 'component'
    )
    const match = component?.initializer
      .getText(catalog)
      .match(/import\(['"]@\/(views\/[^'"]+)['"]\)/)
    if (name && match)
      destinations.set(
        match[1],
        name.initializer.getText(catalog).replace(/^['"]|['"]$/g, '')
      )
  }
  ts.forEachChild(node, collect)
}
collect(catalog)

it('keeps cross-page router operations in navigation while allowing local route state', () => {
  expect(destinations.size).toBe(10)
  expect(readFileSync(resolve(root, 'router/catalog.ts'), 'utf8')).toMatch(
    /name: 'question-list'[\s\S]*?props: \(route\) => parseQuestionListQuery\(route\.query\)/
  )
  for (const path of files(root)) {
    const name = relative(root, path).replaceAll('\\', '/')
    if (name.startsWith('router/')) continue
    const text = readFileSync(path, 'utf8')
    expect(text, name).not.toMatch(/\$(?:router|route)\b/)
    expect(text, name).not.toMatch(
      /['"`]\/(?:home|question|questions|review|profile|settings|dev)(?:[/'"`?#])/
    )
    expect(text, name).not.toMatch(/\buseRouter\b/)
    expect(text, name).not.toMatch(
      /(?:from\s*|import\s*\()['"][^'"]*\/router\/(?:index|navigation)(?:\.ts)?['"]/
    )
    if (
      name === 'views/QuestionListView.vue' ||
      name === 'views/QuestionDetailView.vue'
    ) {
      expect(text, name).not.toMatch(/\buseRoute\b/)
    }
    const shell = name === 'App.vue' || name === 'components/AppNavigation.vue'
    if (!destinations.has(name) && !shell) {
      expect(
        text.replace(/import type[\s\S]*?from ['"]vue-router['"]/g, ''),
        name
      ).not.toMatch(/(?:from\s*|import\s*\()['"]vue-router['"]/)
      continue
    }
    const source = parse(
      name.endsWith('.vue')
        ? [...text.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g)]
            .map((match) => match[1])
            .join('\n')
        : text
    )
    function check(node: ts.Node) {
      if (ts.isCallExpression(node)) {
        if (node.expression.getText(source) === 'useRoute' && !shell) {
          expect(
            node.arguments[0]?.getText(source).replace(/^['"]|['"]$/g, ''),
            name
          ).toBe(destinations.get(name))
        }
      }
      ts.forEachChild(node, check)
    }
    check(source)
  }
})
