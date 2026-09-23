import { expect, type Page, test } from '@playwright/test'

type NativeArgs = { request?: Record<string, unknown> }
declare global {
  interface Window {
    __uiCommands: { command: string; args?: NativeArgs }[]
    __TAURI_INTERNALS__: {
      transformCallback: () => number
      unregisterCallback: () => void
      convertFileSrc: (path: string) => string
      invoke: (command: string, args?: NativeArgs) => Promise<unknown>
    }
  }
}

// UI fixtures only. Native camera, file associations and sharing need device checks.
async function seed(page: Page) {
  await page.addInitScript(() => {
    const at = '2026-09-12T00:00:00Z'
    const subject = {
      id: 'math',
      name: '数学',
      color: '#2563eb',
      createdAt: at,
      updatedAt: at
    }
    const source = {
      id: 'book',
      subjectId: 'math',
      book: '高等数学',
      chapter: '第一章',
      knowledge: '函数与极限',
      createdAt: at,
      updatedAt: at
    }
    const question = {
      id: 'q1',
      stem: '求函数 $f(x) = x^2$ 在 $x=2$ 处的导数。',
      correctAnswer: "$f'(2)=4$",
      explanation: "根据求导法则，$f'(x)=2x$。",
      note: '留意求导后还需要代入。',
      questionType: 'SHORT_ANSWER',
      sourceId: 'book',
      tagIds: ['tag'],
      attachmentIds: [],
      createdAt: at,
      updatedAt: at
    }
    const srs = {
      questionId: 'q1',
      stability: 4,
      difficulty: 5,
      retrievability: 0.5,
      nextReviewAt: at,
      lastReviewAt: at,
      reviewCount: 2,
      isDue: true
    }
    const tags = [{ id: 'tag', name: '计算失误', color: '#7c3aed' }]
    const questions = [question]
    window.__uiCommands = []
    window.__TAURI_INTERNALS__ = {
      transformCallback: () => 1,
      unregisterCallback: () => {},
      convertFileSrc: (path: string) => path,
      invoke: async (command: string, args?: NativeArgs) => {
        window.__uiCommands.push({ command, args })
        const request = args?.request
        switch (command) {
          case 'list_questions':
            return { items: questions, total: questions.length }
          case 'get_question':
            return { question: questions[0] }
          case 'list_subjects':
            return { subjects: [subject] }
          case 'list_sources':
            return { sources: [source] }
          case 'list_tags':
            return { tags }
          case 'list_srs_data':
            return { items: [srs] }
          case 'get_srs_data':
            return { srs }
          case 'get_library_statistics':
            return {
              statistics: {
                questionTotal: questions.length,
                cardTotal: 1,
                dueCount: 1,
                newCardCount: 0,
                averageStability: 4,
                averageDifficulty: 5,
                totalReviews: 2
              }
            }
          case 'submit_review':
            return { srs: { ...srs, reviewCount: 3 }, nextIntervalDays: 7 }
          case 'get_attachments':
            return { attachments: [] }
          case 'get_opened_urls':
            return []
          case 'create_subject':
            return { subject: { ...subject, ...request, id: 'new' } }
          case 'create_source':
            return { source: { ...source, ...request, id: 'new-source' } }
          case 'create_question': {
            const created = { ...question, ...request, id: 'new' }
            questions.push(created)
            return { question: created }
          }
          default:
            if (command.startsWith('plugin:event|')) return 1
            throw new Error('Unmocked native command: ' + command)
        }
      }
    }
  })
}

test.beforeEach(async ({ page }) => {
  await seed(page)
})

for (const width of [360, 1280]) {
  test('centralized navigation and search at ' + width, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 })
    await page.goto('/home')
    await expect(page).toHaveURL(/\/home$/)
    const menu = page.locator(width >= 1024 ? '.q-drawer' : '.mobile-tabs')
    const title = page.locator('.q-toolbar__title')
    const back = page
      .locator('.app-header')
      .getByRole('link', { name: '返回', exact: true })
    await expect(title).toHaveText('首页')
    await expect(back).toHaveCount(0)
    await expect(
      menu.locator(
        width >= 1024
          ? '[aria-label="主导航"] .q-item__label:not(.q-item__label--caption)'
          : '.q-tab__label'
      )
    ).toHaveText(['首页', '添加', '管理', '复习', '我的'])
    const historyBeforeHome = await page.evaluate(() => history.length)
    await menu.getByText('首页', { exact: true }).click()
    await expect(page).toHaveURL(/\/$/)
    expect(await page.evaluate(() => history.length)).toBe(
      historyBeforeHome + 1
    )
    if (width >= 1024) {
      await page.goto('/home')
      const homeItem = menu.locator('.q-item').filter({ hasText: '首页' })
      await homeItem.focus()
      await homeItem.press('Enter')
      await expect(page).toHaveURL(/\/$/)
    }
    await menu.getByText('添加', { exact: true }).click()
    await expect(title).toHaveText('添加错题')
    await expect(back).toHaveCount(0)
    await expect(
      menu.locator(width >= 1024 ? '.nav-active' : '.q-tab--active')
    ).toContainText('添加')
    await menu.getByText('管理', { exact: true }).click()
    await expect(page).toHaveURL(/\/question\/list$/)
    await page.goto('/question/q1')
    await expect(title).toHaveText('错题详情管理')
    await expect(
      menu.locator(width >= 1024 ? '.nav-active' : '.q-tab--active')
    ).toContainText('管理')
    await page
      .locator('.app-header')
      .getByRole('link', { name: '返回', exact: true })
      .click()
    await expect(page).toHaveURL(/\/question\/list$/)
    await menu.getByText('首页', { exact: true }).click()
    await page.getByRole('button', { name: '搜索错题', exact: true }).click()
    const search = page.getByPlaceholder('搜索题干、解析或笔记')
    await expect(search).toBeFocused()
    await expect(page).toHaveURL(/\/question\/list$/)
    await search.fill('保留筛选')
    await page.getByRole('button', { name: '搜索错题', exact: true }).click()
    await expect(search).toBeFocused()
    await expect(page).toHaveURL(/\/question\/list$/)
    await expect(search).toHaveValue('保留筛选')
    await menu.getByText('复习', { exact: true }).click()
    await page.getByRole('button', { name: '开始复习' }).click()
    await expect(page).toHaveURL(/\/review\/session$/)
    await expect(title).toHaveText('复习详情')
    await expect(
      menu.locator(width >= 1024 ? '.nav-active' : '.q-tab--active')
    ).toContainText('复习')
    await page
      .locator('.app-header')
      .getByRole('link', { name: '返回', exact: true })
      .click()
    await expect(page).toHaveURL(/\/review$/)
    await page
      .locator('.app-header')
      .getByRole('link', { name: '设置', exact: true })
      .click()
    await expect(page).toHaveURL(/\/settings$/)
    await page
      .locator('.app-header')
      .getByRole('link', { name: '返回', exact: true })
      .click()
    await expect(page).toHaveURL(/\/$/)
    await page.goto('/settings/sync')
    await expect(title).toHaveText('同步')
    await back.click()
    await expect(page).toHaveURL(/\/settings$/)
    await expect(title).toHaveText('设置')
    await back.click()
    await expect(page).toHaveURL(/\/$/)
    await page.goto('/dev/markdown')
    await expect(title).toHaveText('Markdown 组件测试')
    await back.click()
    await expect(page).toHaveURL(/\/$/)
  })
}

for (const width of [360, 768, 1280]) {
  test('responsive pages at ' + width, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 })
    const errors: string[] = []
    page.on('pageerror', (error) => errors.push(error.message))
    for (const path of [
      '/home',
      '/question/create',
      '/question/list',
      '/review',
      '/profile',
      '/settings',
      '/question/q1',
      '/settings/sync',
      '/dev/markdown'
    ]) {
      await page.goto(path)
      const roots: Record<string, string> = {
        '/home': '.home-page',
        '/question/create': '.add-page',
        '/question/list': '.manage-page',
        '/review': '.preview-page',
        '/profile': '.profile-page',
        '/settings': '.settings-page',
        '/question/q1': '.manage-detail-page',
        '/settings/sync': '.unsupported-page',
        '/dev/markdown': '.markdown-textarea'
      }
      await expect(page.locator(roots[path]).first()).toBeVisible()
      await expect(page.locator('.app-page')).toBeVisible()
      await expect(page.locator('.app-page .loading-state')).toHaveCount(0)
      await expect(page.locator('.app-page')).not.toContainText('加载失败')
      await expect(
        page.locator(width >= 1024 ? '.q-drawer' : '.mobile-tabs')
      ).toBeVisible()
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth > window.innerWidth + 1
      )
      expect(overflow, path + ' must fit the viewport').toBe(false)
      await page.screenshot({
        path:
          'test-results/screens/' +
          width +
          '-' +
          path.replaceAll('/', '_') +
          '.png',
        fullPage: true
      })
    }
    expect(errors).toEqual([])
  })
}

test('mobile filters and dialogs remain usable', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 780 })
  await page.goto('/question/list')
  await page.getByRole('button', { name: '筛选与排序' }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.getByRole('button', { name: '查看结果' }).click()
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await page.getByRole('button', { name: '导出', exact: true }).click()
  await expect(page.getByRole('dialog')).toContainText('导出错题')
  await page
    .getByRole('dialog')
    .getByRole('button', { name: '取消', exact: true })
    .click()
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await page.goto('/question/create')
  await page.getByRole('combobox', { name: '科目', exact: true }).click()
  await page.getByText('添加新科目', { exact: true }).click()
  await expect(page.getByRole('dialog')).toContainText('添加新科目')
  await page
    .getByRole('dialog')
    .getByRole('button', { name: '取消', exact: true })
    .click()
})

test('theme persists and review requires explicit submission', async ({
  page
}) => {
  await page.goto('/settings')
  await page.locator('.theme-select').click()
  await page.getByRole('option', { name: '深色主题' }).click()
  await expect(page.locator('body')).toHaveClass(/body--dark/)
  await page.reload()
  await expect(page.locator('body')).toHaveClass(/body--dark/)
  await page.goto('/review')
  await page.getByRole('button', { name: '开始复习' }).click()
  await page.getByRole('button', { name: '显示答案' }).click()
  await expect(page.getByRole('button', { name: '提交并继续' })).toBeVisible()
  await page.getByRole('slider', { name: '掌握程度' }).press('ArrowRight')
  await expect(page).toHaveURL(/review\/session/)
  await page.getByRole('button', { name: '提交并继续' }).click()
  await expect(page).toHaveURL(/\/review$/)
})

test('source selection, markdown editing and save preserve the question', async ({
  page
}) => {
  await page.goto('/question/create')
  await page.getByRole('combobox', { name: '科目', exact: true }).click()
  await page.getByRole('option', { name: '数学', exact: true }).click()
  await page.getByRole('combobox', { name: '书名', exact: true }).click()
  await page.getByRole('option', { name: '高等数学', exact: true }).click()
  const prompt = page.getByPlaceholder('请输入题目...')
  await prompt.fill('求 $x^2$ 的导数')
  await page.getByPlaceholder('请输入答案...').fill('$2x$')
  await prompt.press('Control+Enter')
  await expect(
    page.locator('.markdown-textarea').first().locator('.katex')
  ).toBeVisible()
  await page.getByRole('button', { name: '保存', exact: true }).click()
  await expect
    .poll(() =>
      page.evaluate(() =>
        window.__uiCommands.some((call) => call.command === 'create_question')
      )
    )
    .toBe(true)
  const request = await page.evaluate(
    () =>
      window.__uiCommands.find((call) => call.command === 'create_question')
        ?.args?.request
  )
  expect(request?.stem).toBe('求 $x^2$ 的导数')
  expect(request?.correctAnswer).toBe('$2x$')
  expect(request?.sourceId).toBe('new-source')
})

test('batch export selection and delete cancellation', async ({ page }) => {
  await page.goto('/question/list')
  await page.getByRole('button', { name: '批量选择' }).click()
  await expect(
    page.getByRole('button', { name: '导出', exact: true })
  ).toBeDisabled()
  await page.getByRole('checkbox', { name: '选择此题' }).click()
  await page.getByRole('button', { name: '导出', exact: true }).click()
  await expect(page.getByRole('dialog')).toContainText('1 道错题')
  await page.getByRole('dialog').getByRole('button', { name: '取消' }).click()
  await page.goto('/question/q1')
  await page.getByRole('button', { name: '删除', exact: true }).click()
  await page
    .getByRole('dialog')
    .getByRole('button', { name: '取消', exact: true })
    .click()
  expect(
    await page.evaluate(() =>
      window.__uiCommands.some((call) => call.command === 'delete_question')
    )
  ).toBe(false)
})

test('system theme changes reach both Quasar and content', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' })
  await page.goto('/home')
  await expect(page.locator('body')).toHaveClass(/body--light/)
  await page.emulateMedia({ colorScheme: 'dark' })
  await expect(page.locator('body')).toHaveClass(/body--dark/)
  await expect(page.locator('body')).toHaveClass(/dark-theme/)
})

test('Markdown sanitization blocks executable HTML and unsafe URLs in the browser', async ({
  page
}) => {
  await page.goto('/home')
  const result = await page.evaluate(async () => {
    const modulePath = '/src/utils/markdown.ts'
    const { renderMarkdown } = await import(/* @vite-ignore */ modulePath)
    const content = document.createElement('div')
    content.innerHTML = renderMarkdown(
      '<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>\n\n[click](javascript:alert%281%29)\n\n$x^2$'
    )
    return {
      executable: !!content.querySelector('script, img, [onerror], [onclick]'),
      href: content.querySelector('a')?.getAttribute('href'),
      math: !!content.querySelector('.katex'),
      text: content.textContent
    }
  })
  expect(result.executable).toBe(false)
  expect(result.href).toBeNull()
  expect(result.math).toBe(true)
  expect(result.text).toContain('<script>')
})

test('mobile image editor can rotate, save and reopen an uploaded image', async ({
  page
}) => {
  await page.setViewportSize({ width: 360, height: 780 })
  await page.goto('/question/create')
  const png = await page.evaluate(() => {
    const canvas = document.createElement('canvas')
    canvas.width = 240
    canvas.height = 160
    const context = canvas.getContext('2d')!
    context.fillStyle = '#ffffff'
    context.fillRect(0, 0, 240, 160)
    context.fillStyle = '#111111'
    context.fillText('x + 2 = 4', 40, 60)
    return canvas.toDataURL('image/png').split(',')[1]
  })
  await page.locator('input[type=file]').setInputFiles({
    name: 'question.png',
    mimeType: 'image/png',
    buffer: Buffer.from(png, 'base64')
  })
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.getByRole('button', { name: '旋转', exact: true }).click()
  await page.getByRole('button', { name: '↻ 右转90°', exact: true }).click()
  await page.getByRole('button', { name: '保存图片编辑' }).click()
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await expect(page.getByAltText('题目图片 1')).toBeVisible()
  await page.getByRole('button', { name: '编辑', exact: true }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.getByRole('button', { name: '取消图片编辑' }).click()
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await expect(page.getByAltText('题目图片 1')).toBeVisible()
})
