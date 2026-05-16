#!/usr/bin/env bash
#
# treegen 安装脚本
# 用法: curl -fsSL https://github.com/luoy-oss/treegen/releases/latest/download/install.sh | bash
#
set -euo pipefail

APP_NAME="treegen"
REPO="luoy-oss/treegen"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

# 检测操作系统和架构
detect_platform() {
    local os
    local arch

    case "$(uname -s)" in
        Linux)  os="unknown-linux-gnu" ;;
        Darwin) os="apple-darwin" ;;
        *)
            echo "不支持的操作系统: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            echo "不支持的架构: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${arch}-${os}"
}

# 获取下载 URL
get_download_url() {
    local platform="$1"
    curl -fsSL "$API_URL" | grep -o "https://[^\"]*${platform}\.tar\.gz" | head -1
}

# 主流程
main() {
    echo "正在安装 ${APP_NAME}..."

    local platform
    platform=$(detect_platform)
    echo "检测到平台: ${platform}"

    local download_url
    download_url=$(get_download_url "$platform")
    if [ -z "$download_url" ]; then
        echo "错误: 未能找到 ${platform} 的下载包"
        exit 1
    fi

    local tmp_dir
    tmp_dir=$(mktemp -d)
    cd "$tmp_dir"

    echo "下载中: ${download_url}"
    curl -fsSL "$download_url" -o "${APP_NAME}.tar.gz"

    echo "解压中..."
    tar xzf "${APP_NAME}.tar.gz"

    # 安装到 /usr/local/bin
    local install_dir="/usr/local/bin"
    if [ ! -w "$install_dir" ]; then
        echo "需要管理员权限安装到 ${install_dir}"
        sudo mv "${APP_NAME}" "${install_dir}/"
    else
        mv "${APP_NAME}" "${install_dir}/"
    fi

    chmod +x "${install_dir}/${APP_NAME}"

    # 清理
    cd /
    rm -rf "$tmp_dir"

    echo ""
    echo "${APP_NAME} 安装成功!"
    echo "运行以下命令开始使用:"
    echo "  ${APP_NAME} --help"
}

main
