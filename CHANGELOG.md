# 更新说明

## 0.1.1

### 新增

- GitHub 仓库文章支持自动获取更完整的信息：
  - Stars、Forks、Watchers、Open Issues
  - 语言、许可证、默认分支、Topics、主页
  - 创建时间、更新时间、归档状态
  - 自动抓取并展示仓库 README，支持表格、代码、折叠详情等 Markdown 内容
- 转录增强：
  - 设置中新增“转录后自动使用 AI 加标点 / 校对”
  - 转录失败或没有音频时，会依次尝试本地同目录字幕和 Bilibili 字幕
  - 播放页保留手动 AI 校对入口
- 新手引导：
  - 首次启动展示功能引导
  - 设置页支持重新打开
- 检查更新：
  - 关于页查询 GitHub Releases 最新版本
  - 显示版本号、更新说明和下载入口
- 跨平台构建：
  - Windows NSIS / MSI
  - Linux deb / AppImage
  - macOS Intel / Apple Silicon
  - Android arm64 Debug APK

### 修复

- 修复 release 版本中彩色卡片、块和字体颜色全部变成白色/黑色的问题：
  - 禁用 Tauri 对 `style-src` 的 nonce 注入
  - 允许 Vuetify 运行时主题 CSS 变量
- 修复 Bilibili 旧封面 `http://` 地址被 CSP 拦截的问题
- 修复 Bilibili 弹幕 raw DEFLATE 解压失败，导致弹幕不显示的问题
- 修复视频页“浏览器打开”和“新窗口播放”图标重复
- 修复侧栏滚动、标签入口和收起按钮显示问题
- 修复视频库分类栏过窄、长分类名截断的问题
- 增加分类栏固定/悬停展开模式
- 修复统计页顶部卡片高度和下方内容溢出
- 修复首页最近添加视频卡片底部空隙过大
- 优化播放器加载：
  - 视频默认在主窗口播放
  - 评论按需加载
  - 弹幕和字幕延迟加载
  - 移动端限制同屏弹幕数量

### 构建

- 更新版本号到 `0.1.1`
- Android versionCode 更新为 `1001`
- 修复 GitHub Actions Android NDK/SDK 安装流程
- 修复跨平台构建中的前端类型检查问题

Windows 本地构建：

```powershell
npm run tauri build -- --bundles nsis,msi
```

跨平台安装包由 `.github/workflows/build-desktop.yml` 生成。
