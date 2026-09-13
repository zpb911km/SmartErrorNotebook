# 前端 UI 架构

前端保留 Vue 3、Vue Router、Vite 与 Tauri 2，通过 Quasar Vite 插件按需接入组件，不另建 Quasar CLI 工程。现有数据库模型、IPC 合约、复习算法以及本地数据服务不随 UI 改写。

## 分层与约定

- `App.vue` 与 `AppNavigation.vue`：唯一的 QLayout 应用壳，负责页面容器、安全区域与导航。宽度达到 1024px 时显示侧栏，较小屏幕显示底部导航。详情页使用明确的父页面返回入口。
- `quasar.ts`、`main.ts`：集中注册中文语言包、Dialog、Notify、图标与基础样式。不要在页面重复安装插件。
- `views/`：编排页面状态与业务操作。可复用交互放在 `components/`，持久化仍经过 `services/` 和 `api/`，不在展示组件直接拼接 IPC。
- `LibraryFilters.vue`：管理页与复习页共用的筛选字段；`ResponsiveFilterPanel.vue` 只负责桌面面板/移动弹窗。来源层级由 `sourceCatalog`、`sourceSelection` 统一处理。
- `SourceSelector.vue`、`ErrorTagSelector.vue`：编辑本地草稿；来源的落库由保存边界完成，避免异步选择状态和保存互相竞争。
- `useTheme.ts`：持久化主题偏好，使用 Quasar Dark 作为实际明暗状态来源；兼容内容区现有 CSS 变量，不重复监听系统主题。
- `utils/dialog.ts`、`utils/notification.ts`：统一确认与通知入口。危险操作必须确认，异步提交必须防重入，并保留失败时的草稿。

共享布局和主题变量分别在 `styles/global.css`、`styles/theme.css`。组件私有样式使用 scoped。不要重定义 Quasar 的 `text-secondary` 等工具类，应用自己的辅助色使用 `text-muted`。

## 内容与异步安全

所有 Markdown 预览和 HTML/PDF 导出共用 `utils/markdown.ts`：独立 Marked 实例、局部语言注册、KaTeX `trust: false`，最终经过 DOMPurify。原始 HTML 作为文本展示；不提供绕过清理的回退。数学分隔符转换不得改写代码内容。

此渲染入口需要浏览器/WebView DOM，不是可直接用于服务端 Node 的纯函数。happy-dom 测试仅覆盖渲染逻辑，安全断言在真实浏览器执行。

读取数据使用请求版本或 `useLatestRequest` 忽略旧响应。AI 识别不会覆盖请求期间的手动修改，也不会把重置前的结果写回新表单；失败结果不清空字段。请求失效只阻止结果应用，不代表已取消远端请求或计费。

相机流、图片拖拽监听、图片编辑延迟任务在弹窗关闭/卸载时释放；延迟获得的相机流也必须检查当前会话后使用。复习评分由 QSlider 输入，并通过独立按钮提交，不在拖动结束时自动写入。

## 验证

```sh
pnpm install --frozen-lockfile
pnpm lint
pnpm test
pnpm build
pnpm test:ui
```

`pnpm lint` 依次运行 ESLint 与 Prettier 格式检查，任一步失败都会返回非零退出码。`pnpm format` 统一格式化源码、测试以及根目录 TypeScript/JSON 配置；`pnpm format:check` 仅检查同一范围。

Playwright 在 Windows 使用已安装的 Microsoft Edge；其他系统先运行 `pnpm exec playwright install chromium`。测试通过 Node 直接启动本地 Vite 服务，固定端口 1420，测试结束时释放服务；本地已有服务时会复用且不会将其关闭，运行前应确认它对应当前仓库。CI 不复用已有服务。

Windows 受限沙箱中，服务器进程树清理可能挂起，即使用例已全部通过。遇到此情况应在具备进程清理权限的环境复测，或在本地预先启动当前仓库的 Vite 供测试复用；验收需确认测试进程以退出码 0 正常结束。

浏览器测试覆盖 360/768/1280px 布局、主要路由无横向溢出、移动筛选和弹窗、主题持久化/跟随系统、录入来源与 Markdown 保存、图片上传/旋转/保存、批量导出选择、删除取消、复习显式提交以及 Markdown 清理。截图输出到忽略提交的 `test-results/screens/`。

浏览器测试通过模拟 Tauri IPC 使用固定数据，不访问真实数据库或外部 AI 服务，不能替代原生端验收。发布前仍需在桌面 Tauri 与 Android 真机验证：相机授权/取消、图片编辑与长图、文件导入及文件关联、导出保存/分享、系统返回键、安全区域和软键盘。远程同步和社区仍保持停用；`/markdown-test` 是开发调试入口。

Quasar 版本固定在满足仓库依赖发布时间策略的版本；升级时保留锁文件与供应链检查，不添加绕过发布时间检查的例外。
