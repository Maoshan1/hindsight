<p align="center">
  <img src="src-tauri/icons/icon-512.png" width="120" alt="Hindsight Logo">
</p>

<h1 align="center">Hindsight</h1>

<p align="center">
  <strong>终端历史的「后见之明」</strong><br>
  <em>再也不用担心找不到之前写过的命令。</em>
</p>

<p align="center">
  <a href="#功能特性">功能</a> •
  <a href="#安装">安装</a> •
  <a href="#使用方法">使用</a> •
  <a href="#对比">对比</a> •
  <a href="#贡献">贡献</a>
</p>

---

## 解决什么问题？

三周前写过一条很长的 ffmpeg 命令，今天又需要了。但你完全不记得具体参数是什么。

`Ctrl+R` 只能按前缀搜索，你不记得命令的开头，只记得它跟 `ffmpeg` 和 `scale` 有关。

**Hindsight 解决这个问题。** 它自动记录每条命令的上下文（工作目录、退出码、耗时），让你可以用任意关键词搜索。

## 功能特性

- **全文搜索** — 输入命令里的任意关键词就能找到
- **上下文感知** — 显示每条命令是在哪个目录执行的
- **失败命令筛选** — 快速找到出错的命令
- **隐私优先** — 100% 本地存储，无需云端，无需账号
- **非侵入式** — 与现有 history 共存，不替换任何东西
- **macOS GUI** — 美观的菜单栏应用，支持模糊搜索
- **CLI 工具** — 适合终端用户的快速命令行版本

## 安装

### macOS 应用（推荐）

```bash
# 从 Releases 下载 DMG
# https://github.com/Maoshan1/hindsight/releases

# 或从源码构建
git clone https://github.com/Maoshan1/hindsight.git
cd hindsight
npm install
cargo install tauri-cli
npm run tauri build
```

### CLI（Homebrew）

```bash
brew tap Maoshan1/hindsight
brew install Maoshan1/hindsight/hindsight
```

### CLI（Cargo）

```bash
cargo install hindsight
```

## 配置

在 `~/.zshrc`（或 `~/.bashrc`）中添加：

```bash
# Homebrew 安装
source $(brew --prefix)/share/hindsight/hindsight.zsh

# Cargo 安装
source ~/.hindsight/hindsight.zsh
```

之后你执行的每条命令都会自动记录。

## 使用方法

### GUI 应用

点击菜单栏图标打开搜索窗口，输入关键词即可搜索。

| 操作 | 快捷键 |
|------|--------|
| 打开搜索 | 点击菜单栏图标 |
| 上下选择 | `↑` / `↓` |
| 复制命令 | 选中后按 `Enter` |
| 关闭窗口 | 点击红色按钮（应用保持在后台） |
| 彻底退出 | 右键菜单栏图标 → Quit |

### CLI

```bash
# 搜索历史
hindsight ffmpeg
hindsight "docker compose"

# 只搜索失败的命令
hindsight --exit 1

# 最常用的命令
hindsight top

# 最近的命令
hindsight
```

## 工作原理

```
┌─────────────────────────────────────────────────┐
│  你的 Shell（zsh/bash）                          │
│  preexec/precmd hooks → 记录命令 + 元数据        │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  SQLite 数据库（~/.hindsight/history.db）         │
│  - 完整命令文本                                   │
│  - 工作目录                                       │
│  - 退出码和耗时                                   │
│  - FTS5 全文搜索索引                              │
└──────────────────────┬──────────────────────────┘
                       │
              ┌────────┴────────┐
              ▼                 ▼
        ┌──────────┐    ┌──────────┐
        │  GUI 应用 │    │   CLI    │
        │ (Tauri)  │    │ (Rust)   │
        └──────────┘    └──────────┘
```

## 对比

| 功能 | **Hindsight** | atuin | mcfly |
|------|:------------:|:-----:|:-----:|
| 非侵入式（保留现有 history） | ✅ | ❌ | ❌ |
| 全文搜索 | ✅ | ✅ | ❌ |
| 按目录筛选 | ✅ | ✅ | ❌ |
| 按退出码筛选 | ✅ | ✅ | ❌ |
| macOS GUI 应用 | ✅ | ❌ | ❌ |
| 隐私（无需云端） | ✅ | ❌ | ✅ |
| 无需 daemon 即可使用 | ✅ | ❌ | ❌ |
| 云端同步 | 🔜 | ✅ | ❌ |
| 团队共享历史 | 🔜 | ✅ | ❌ |

**核心区别：** Hindsight 不替换你的 shell history 系统。它只是在旁边做增强 — 零风险、零迁移、零配置修改。

## 路线图

- [x] CLI 全文搜索
- [x] macOS GUI 应用
- [x] Shell hooks（zsh、bash）
- [ ] 云端同步（可选）
- [ ] 团队共享历史
- [ ] AI 语义搜索
- [ ] Linux 支持
- [ ] Windows 支持

## 贡献

欢迎贡献代码！欢迎提交 Issue 或 Pull Request。

```bash
# 开发环境
git clone https://github.com/Maoshan1/hindsight.git
cd hindsight
npm install
npm run tauri dev
```

## 许可证

[MIT](LICENSE)
