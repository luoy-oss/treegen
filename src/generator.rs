use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::FsNode;

/// 生成选项
#[derive(Debug)]
pub struct GenerateOptions {
    /// 输出根目录
    pub output_dir: PathBuf,
    /// 是否覆盖已存在的文件
    pub overwrite: bool,
    /// 仅执行空运行（不实际创建）
    pub dry_run: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            overwrite: false,
            dry_run: false,
        }
    }
}

/// 生成结果统计
#[derive(Debug, Default)]
pub struct GenerateResult {
    pub dirs_created: usize,
    pub files_created: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

/// 根据解析后的树形节点列表生成文件结构
pub fn generate(nodes: &[FsNode], options: &GenerateOptions) -> anyhow::Result<GenerateResult> {
    let mut result = GenerateResult::default();

    for node in nodes {
        generate_node(node, &options.output_dir, options, &mut result)?;
    }

    Ok(result)
}

/// 递归生成单个节点
fn generate_node(
    node: &FsNode,
    parent_path: &Path,
    options: &GenerateOptions,
    result: &mut GenerateResult,
) -> anyhow::Result<()> {
    let node_path = parent_path.join(node.name());

    match node {
        FsNode::Dir(_name, children) => {
            if options.dry_run {
                println!("[DRY-RUN] 创建目录: {}", node_path.display());
                result.dirs_created += 1;
            } else {
                if node_path.exists() {
                    if node_path.is_dir() {
                        // 目录已存在，正常继续
                    } else {
                        let msg = format!("路径已存在但不是目录: {}", node_path.display());
                        result.errors.push(msg.clone());
                        anyhow::bail!(msg);
                    }
                } else {
                    fs::create_dir_all(&node_path).map_err(|e| {
                        anyhow::anyhow!("创建目录失败 '{}': {}", node_path.display(), e)
                    })?;
                    println!("创建目录: {}", node_path.display());
                    result.dirs_created += 1;
                }
            }

            // 递归生成子节点
            for child in children {
                generate_node(child, &node_path, options, result)?;
            }
        }
        FsNode::File(_name) => {
            if options.dry_run {
                println!("[DRY-RUN] 创建文件: {}", node_path.display());
                result.files_created += 1;
                return Ok(());
            }

            if node_path.exists() {
                if node_path.is_file() {
                    if options.overwrite {
                        fs::write(&node_path, "").map_err(|e| {
                            anyhow::anyhow!("写入文件失败 '{}': {}", node_path.display(), e)
                        })?;
                        println!("覆盖文件: {}", node_path.display());
                        result.files_created += 1;
                    } else {
                        println!(
                            "跳过已存在文件: {} (使用 --overwrite 覆盖)",
                            node_path.display()
                        );
                        result.skipped += 1;
                    }
                } else {
                    let msg = format!("路径已存在但不是文件: {}", node_path.display());
                    result.errors.push(msg.clone());
                    anyhow::bail!(msg);
                }
            } else {
                // 确保父目录存在
                if let Some(parent) = node_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| {
                            anyhow::anyhow!("创建父目录失败 '{}': {}", parent.display(), e)
                        })?;
                    }
                }
                fs::write(&node_path, "").map_err(|e| {
                    anyhow::anyhow!("创建文件失败 '{}': {}", node_path.display(), e)
                })?;
                println!("创建文件: {}", node_path.display());
                result.files_created += 1;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dry_run() {
        let nodes = vec![FsNode::Dir(
            "test_project".to_string(),
            vec![
                FsNode::File("README.md".to_string()),
                FsNode::Dir("src".to_string(), vec![FsNode::File("main.ts".to_string())]),
            ],
        )];

        let options = GenerateOptions {
            output_dir: PathBuf::from("."),
            overwrite: false,
            dry_run: true,
        };

        let result = generate(&nodes, &options).unwrap();
        assert_eq!(result.dirs_created, 2); // test_project, src
        assert_eq!(result.files_created, 2); // README.md, main.ts
        assert_eq!(result.errors.len(), 0);
    }
}
