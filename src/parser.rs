use std::fmt;

/// 文件系统节点：可以是文件或目录
#[derive(Debug, Clone)]
pub enum FsNode {
    File(String),
    Dir(String, Vec<FsNode>),
}

#[allow(dead_code)]
impl FsNode {
    pub fn name(&self) -> &str {
        match self {
            FsNode::File(name) | FsNode::Dir(name, _) => name,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, FsNode::Dir(_, _))
    }

    pub fn children(&self) -> &[FsNode] {
        match self {
            FsNode::Dir(_, children) => children,
            FsNode::File(_) => &[],
        }
    }
}

impl fmt::Display for FsNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsNode::File(name) => write!(f, "{}", name),
            FsNode::Dir(name, children) => {
                writeln!(f, "{}/", name)?;
                let last_idx = children.len().saturating_sub(1);
                for (i, child) in children.iter().enumerate() {
                    let prefix = if i == last_idx {
                        "└── "
                    } else {
                        "├── "
                    };
                    let child_str = child.to_string();
                    for (j, line) in child_str.lines().enumerate() {
                        if j == 0 {
                            writeln!(f, "{}{}", prefix, line)?;
                        } else {
                            let cont = if i == last_idx { "    " } else { "│   " };
                            writeln!(f, "{}{}", cont, line)?;
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

/// 解析后的条目：包含深度、名称和是否为目录
#[derive(Debug)]
struct Entry {
    depth: usize,
    name: String,
    is_dir: bool,
}

/// 从一行文本中提取深度和名称
fn parse_line(line: &str) -> Option<Entry> {
    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

    if trimmed.is_empty() {
        return None;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let len = chars.len();
    let mut pos = 0;

    // 跳过连续的 4 字节目录延续前缀："│   " 或 "    "
    while pos + 4 <= len {
        let c0 = chars[pos];
        if (c0 == '│' || c0 == ' ')
            && chars[pos + 1] == ' '
            && chars[pos + 2] == ' '
            && chars[pos + 3] == ' '
        {
            pos += 4;
            continue;
        }
        break;
    }

    // 跳过分支标记："├── " 或 "└── "
    if pos + 4 <= len {
        let c0 = chars[pos];
        if (c0 == '├' || c0 == '└')
            && chars[pos + 1] == '─'
            && chars[pos + 2] == '─'
            && chars[pos + 3] == ' '
        {
            pos += 4;
        }
    }

    // 没有提取到任何内容（纯前缀行，如单独的 "│"）
    if pos >= len
        || pos == 0
            && chars
                .iter()
                .any(|&c| c == '│' || c == '├' || c == '└' || c == '─')
    {
        return None;
    }

    let name: String = chars[pos..].iter().collect::<String>().trim().to_string();
    if name.is_empty() {
        return None;
    }

    let depth = pos / 4;
    let is_dir = name.ends_with('/');

    Some(Entry {
        depth,
        name: name.trim_end_matches('/').to_string(),
        is_dir,
    })
}

/// 解析树形结构文本，返回根节点列表
pub fn parse_tree(input: &str) -> anyhow::Result<Vec<FsNode>> {
    let entries: Vec<Entry> = input.lines().filter_map(parse_line).collect();

    if entries.is_empty() {
        anyhow::bail!("没有找到有效的树形结构内容，请检查输入格式");
    }

    // 检查第一个条目的深度是否为 0
    if entries[0].depth != 0 {
        anyhow::bail!("根节点深度必须为 0，请检查格式（第一行不应有缩进前缀）");
    }

    build_tree(entries)
}

/// 构建树形结构时的中间节点
struct NodeIndex {
    name: String,
    is_dir: bool,
    children: Vec<usize>,
}

/// 根据深度列表构建树形结构
fn build_tree(entries: Vec<Entry>) -> anyhow::Result<Vec<FsNode>> {
    // 使用中间节点列表来避免借用问题
    let mut nodes: Vec<NodeIndex> = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (depth, node_index)
    let mut roots: Vec<usize> = Vec::new();

    for entry in entries {
        let node_idx = nodes.len();
        nodes.push(NodeIndex {
            name: entry.name.clone(),
            is_dir: entry.is_dir,
            children: Vec::new(),
        });

        if entry.depth == 0 {
            stack.clear();
            roots.push(node_idx);
            stack.push((0, node_idx));
            continue;
        }

        // 弹出栈中深度 >= 当前深度的节点
        while let Some(&(top_depth, _)) = stack.last() {
            if top_depth >= entry.depth {
                stack.pop();
            } else {
                break;
            }
        }

        // 栈顶元素是父节点
        if let Some(&(_, parent_idx)) = stack.last() {
            nodes[parent_idx].children.push(node_idx);
        } else {
            roots.push(node_idx);
        }

        if entry.is_dir {
            stack.push((entry.depth, node_idx));
        }
    }

    // 将中间节点转换为 FsNode
    Ok(convert_nodes(&nodes, &roots))
}

/// 将中间节点列表转换为最终的 FsNode 树
fn convert_nodes(nodes: &[NodeIndex], indices: &[usize]) -> Vec<FsNode> {
    indices
        .iter()
        .map(|&idx| convert_node(nodes, idx))
        .collect()
}

fn convert_node(nodes: &[NodeIndex], idx: usize) -> FsNode {
    let node = &nodes[idx];
    if node.is_dir {
        let children: Vec<FsNode> = node
            .children
            .iter()
            .map(|&child_idx| convert_node(nodes, child_idx))
            .collect();
        FsNode::Dir(node.name.clone(), children)
    } else {
        FsNode::File(node.name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tree() {
        let input = "my-project/\n├── src/\n│   ├── index.ts\n│   └── main.ts\n├── package.json\n└── README.md\n";
        let nodes = parse_tree(input).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name(), "my-project");
        assert!(nodes[0].is_dir());
        let children = nodes[0].children();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].name(), "src");
        assert!(children[0].is_dir());
    }

    #[test]
    fn test_parse_rootless_tree() {
        let input = "├── src/\n│   └── main.ts\n├── package.json\n";
        let result = parse_tree(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_input() {
        let input = "";
        let result = parse_tree(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multi_root() {
        let input = "src/\n├── index.ts\n├── utils/\n│   └── helper.ts\ndist/\n├── output.js\n";
        let nodes = parse_tree(input).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].name(), "src");
        assert_eq!(nodes[1].name(), "dist");
    }

    #[test]
    fn test_display_roundtrip() {
        let input = "my-project/\n├── src/\n│   ├── components/\n│   │   ├── GlassCard.vue\n│   │   ├── StatsDashboard.vue\n│   │   └── CommandPanel.vue\n│   ├── types/\n│   │   └── index.ts\n│   ├── composables/\n│   │   └── useLiveStats.ts\n│   ├── App.vue\n│   └── main.ts\n├── index.html\n├── package.json\n├── tsconfig.json\n├── vite.config.ts\n└── README.md\n";
        let nodes = parse_tree(input).unwrap();
        let output = nodes
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("");
        let output = output.trim().to_string() + "\n";
        assert_eq!(output, input);
    }
}
