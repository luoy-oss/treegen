# treegen

从树形结构文本快速生成目录和文件的命令行工具。

## 简介

在使用 AI 编程时，经常会得到类似 `tree` 命令输出的项目结构描述文本。手动按照这种结构逐个创建目录和文件非常繁琐。`treegen` 专为解决这个问题而设计 -- 它能够解析树形结构文本，并自动在磁盘上生成对应的目录结构和空文件。

例如，给定以下输入：

```
my-project/
├── src/
│   ├── components/
│   │   ├── GlassCard.vue
│   │   ├── StatsDashboard.vue
│   │   └── CommandPanel.vue
│   ├── types/
│   │   └── index.ts
│   ├── composables/
│   │   └── useLiveStats.ts
│   ├── App.vue
│   └── main.ts
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

`treegen` 会自动创建 `my-project/` 目录及其下所有子目录和空文件。

## 安装

### 方式一：通过 Cargo 安装（需要安装 Rust 工具链）

```bash
cargo install treegen
```

### 方式二：Linux / macOS 一键安装

```bash
curl -fsSL https://github.com/luoy-oss/treegen/releases/latest/download/install.sh | bash
```

### 方式三：Windows PowerShell 一键安装

```powershell
irm https://github.com/luoy-oss/treegen/releases/latest/download/install.ps1 | iex
```

### 方式四：从 GitHub Releases 下载

访问 [GitHub Releases 页面](https://github.com/luoy-oss/treegen/releases) 下载对应平台的压缩包，解压后将二进制文件添加到系统 PATH 中。

## 使用方法

### 从标准输入读取

```bash
echo "my-project/\n├── src/\n│   └── main.ts\n└── README.md" | treegen
```

### 从文件读取

将树形结构保存到文件后，直接传参读取：

```bash
treegen structure.txt
```

### 指定输出目录

默认情况下，文件会生成在当前工作目录。使用 `-o` 或 `--output` 指定输出目录：

```bash
treegen structure.txt -o ./my-app
```

### 空运行（仅预览，不实际创建文件）

使用 `--dry-run` 参数可以先预览将要创建的文件结构，确认无误后再实际执行：

```bash
treegen structure.txt --dry-run
```

输出示例：

```
[DRY-RUN] 创建目录: ./my-project
[DRY-RUN] 创建目录: ./my-project/src
[DRY-RUN] 创建目录: ./my-project/src/components
[DRY-RUN] 创建文件: ./my-project/src/components/GlassCard.vue
[DRY-RUN] 创建文件: ./my-project/src/components/StatsDashboard.vue
...
[DRY-RUN] 计划创建: 5 个目录, 12 个文件
```

### 覆盖已存在的文件

默认情况下，如果文件已存在，`treegen` 会跳过并显示提示。使用 `--overwrite` 强制覆盖：

```bash
treegen structure.txt --overwrite
```

### 调试模式：打印解析后的树

使用 `--print-tree` 可以查看 treegen 解析后的内部结构，便于排查解析问题：

```bash
treegen structure.txt --print-tree
```

## 命令行参数参考

```
Usage: treegen [OPTIONS] [INPUT]

Arguments:
  [INPUT]  包含树形结构的输入文件路径（不提供则从标准输入读取）

Options:
  -o, --output <OUTPUT>  输出目录 [默认: .]
      --overwrite        覆盖已存在的文件
      --dry-run          仅执行空运行，不实际创建文件
      --print-tree       显示解析后的树形结构（调试用）
  -h, --help             打印帮助信息
  -V, --version          打印版本号
```

## 输入格式说明

`treegen` 解析类似 `tree` 命令输出的树形结构文本，支持以下前缀字符：

| 前缀 | 含义 |
|------|------|
| `├── ` | 中间兄弟节点 |
| `└── ` | 最后兄弟节点 |
| `│   ` | 父级目录延续 |
| `    ` | 父级目录结束（4个空格） |

规则说明：
- 每 4 个字符为一个缩进层级
- 目录名以 `/` 结尾（如 `src/`）
- 文件名没有尾部斜杠（如 `main.ts`）
- 第一行必须是根节点，不能有缩进前缀
- 支持多个根节点并列

## 实际示例

### 快速创建 Vue 项目结构

```bash
# 将结构保存到文件
cat > vue-project.txt << 'EOF'
my-vue-app/
├── src/
│   ├── components/
│   │   ├── Header.vue
│   │   ├── Sidebar.vue
│   │   └── Footer.vue
│   ├── views/
│   │   ├── Home.vue
│   │   └── About.vue
│   ├── router/
│   │   └── index.ts
│   ├── store/
│   │   └── index.ts
│   ├── App.vue
│   └── main.ts
├── public/
│   └── favicon.ico
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
EOF

# 生成目录结构
treegen vue-project.txt
```

### 结合 AI 使用

从 AI 获得树形结构输出后，可以直接通过管道传递给 `treegen`：

```bash
# 在 AI 对话中复制结构后，粘贴到 echo 或直接保存为文件
pbpaste | treegen           # macOS 粘贴板
xclip -o | treegen          # Linux 粘贴板
```

## 从源码构建

```bash
git clone https://github.com/luoy-oss/treegen.git
cd treegen
cargo build --release
```

编译后的二进制文件位于 `target/release/treegen`（Windows 下为 `target/release/treegen.exe`）。

## 技术栈

- 编程语言：Rust
- 命令行参数解析：clap (derive API)
- 错误处理：anyhow
- 跨平台支持：Windows、Linux、macOS

## 许可证

MIT
