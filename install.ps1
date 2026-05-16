# treegen 安装脚本 (PowerShell)
# 用法: irm https://github.com/luoy-oss/treegen/releases/latest/download/install.ps1 | iex
#
param(
    [string]$InstallDir = "$env:USERPROFILE\.treegen\bin"
)

$AppName = "treegen"
$Repo = "luoy-oss/treegen"
$ApiUrl = "https://api.github.com/repos/${Repo}/releases/latest"

# 检测架构
function Get-Arch {
    $arch = (Get-CimInstance Win32_Processor).Architecture
    switch ($arch) {
        0  { return "x86" }       # x86
        9  { return "x86_64" }    # x64
        12 { return "aarch64" }   # ARM64
        default { throw "不支持的架构: $arch" }
    }
}

function Get-Target {
    $arch = Get-Arch
    switch ($arch) {
        "x86_64"  { return "x86_64-pc-windows-msvc" }
        "aarch64" { return "aarch64-pc-windows-msvc" }
        "x86"     { return "i686-pc-windows-msvc" }
        default   { throw "不支持的架构: $arch" }
    }
}

# 检查是否在 PATH 中已有安装目录
function Ensure-InPath {
    param([string]$Dir)

    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*${Dir}*") {
        $newPath = "${currentPath};${Dir}"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        # 更新当前会话的 PATH
        $env:Path = "${env:Path};${Dir}"
        Write-Host "已添加 ${Dir} 到用户 PATH"
    }
}

# 获取下载 URL
function Get-DownloadUrl {
    param([string]$Target)

    try {
        $release = Invoke-RestMethod -Uri $ApiUrl -UseBasicParsing
        $asset = $release.assets | Where-Object { $_.name -like "*${Target}*" -and $_.name -like "*.zip" } | Select-Object -First 1
        if (-not $asset) {
            $asset = $release.assets | Where-Object { $_.name -like "*${Target}*" -and $_.name -like "*.tar.gz" } | Select-Object -First 1
        }
        return $asset.browser_download_url
    } catch {
        Write-Error "获取发布信息失败: $_"
        exit 1
    }
}

# 主流程
function Main {
    Write-Host "正在安装 ${AppName}..." -ForegroundColor Cyan

    $target = Get-Target
    Write-Host "检测到平台: ${target}" -ForegroundColor Green

    $downloadUrl = Get-DownloadUrl -Target $target
    if (-not $downloadUrl) {
        Write-Error "错误: 未能找到 ${target} 的下载包"
        exit 1
    }

    Write-Host "下载中: ${downloadUrl}" -ForegroundColor Yellow

    $tmpDir = Join-Path $env:TEMP "treegen_install_$(Get-Random)"
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    $zipPath = Join-Path $tmpDir "${AppName}.zip"
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing
    } catch {
        Write-Error "下载失败: $_"
        exit 1
    }

    # 解压
    Write-Host "解压中..." -ForegroundColor Yellow
    $extractPath = Join-Path $tmpDir "extracted"
    New-Item -ItemType Directory -Path $extractPath -Force | Out-Null

    if ($downloadUrl -like "*.zip") {
        Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force
        $binary = Get-ChildItem -Path $extractPath -Recurse -Filter "${AppName}.exe" | Select-Object -First 1
    } else {
        # tar.gz
        tar xzf $zipPath -C $extractPath
        $binary = Get-ChildItem -Path $extractPath -Recurse -Filter "${AppName}*" | Where-Object { -not $_.PSIsContainer } | Select-Object -First 1
    }

    if (-not $binary) {
        Write-Error "错误: 解压后未找到 ${AppName}.exe"
        exit 1
    }

    # 创建安装目录
    $installDir = $InstallDir
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null

    # 复制二进制文件
    Copy-Item -Path $binary.FullName -Destination (Join-Path $installDir "${AppName}.exe") -Force

    # 添加到 PATH
    Ensure-InPath -Dir $installDir

    # 清理
    Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue

    Write-Host ""
    Write-Host "${AppName} 安装成功!" -ForegroundColor Green
    Write-Host "请重新打开终端或运行以下命令刷新 PATH:" -ForegroundColor Yellow
    Write-Host "  \$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User')" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "运行以下命令开始使用:" -ForegroundColor White
    Write-Host "  ${AppName} --help" -ForegroundColor Cyan
}

Main
