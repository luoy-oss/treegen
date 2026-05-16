mod generator;
mod parser;

use std::io::Read;
use std::path::PathBuf;

use clap::Parser;

/// treegen - 从树形结构文本快速生成目录和文件
///
/// 通过解析类似 `tree` 命令输出的树形结构文本，
/// 自动创建对应的目录结构和空文件。
///
/// 使用示例:
///   echo "my-project/\n├── src/\n│   └── main.ts\n└── README.md" | treegen
///   treegen input.txt
///   treegen input.txt -o ./output
///   treegen input.txt --dry-run
#[derive(Parser, Debug)]
#[command(name = "treegen", version, author, about)]
struct Cli {
    /// 包含树形结构的输入文件路径
    /// 如果不提供，则从标准输入读取
    input: Option<PathBuf>,

    /// 输出目录（默认为当前目录）
    #[arg(short, long, default_value = ".")]
    output: PathBuf,

    /// 覆盖已存在的文件
    #[arg(long)]
    overwrite: bool,

    /// 仅执行空运行，不实际创建文件
    #[arg(long)]
    dry_run: bool,

    /// 显示解析后的树形结构（调试用）
    #[arg(long)]
    print_tree: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 读取输入
    let input = read_input(cli.input)?;

    // 解析树形结构
    let nodes = parser::parse_tree(&input)?;

    // 调试模式下打印解析结果
    if cli.print_tree {
        println!("=== 解析结果 ===");
        for node in &nodes {
            print!("{}", node);
        }
        println!("================");
    }

    // 生成文件结构
    let options = generator::GenerateOptions {
        output_dir: cli.output,
        overwrite: cli.overwrite,
        dry_run: cli.dry_run,
    };

    let result = generator::generate(&nodes, &options)?;

    // 输出统计信息
    println!();
    if cli.dry_run {
        println!(
            "[DRY-RUN] 计划创建: {} 个目录, {} 个文件",
            result.dirs_created, result.files_created
        );
    } else {
        println!(
            "完成: 创建 {} 个目录, {} 个文件, 跳过 {} 个已存在文件",
            result.dirs_created, result.files_created, result.skipped
        );
    }

    if !result.errors.is_empty() {
        eprintln!("\n错误:");
        for err in &result.errors {
            eprintln!("  - {}", err);
        }
    }

    Ok(())
}

/// 从文件或标准输入读取内容
fn read_input(path: Option<PathBuf>) -> anyhow::Result<String> {
    match path {
        Some(path) => {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("读取文件失败 '{}': {}", path.display(), e))?;
            Ok(content)
        }
        None => {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|e| anyhow::anyhow!("读取标准输入失败: {}", e))?;
            if input.trim().is_empty() {
                anyhow::bail!("标准输入为空，请通过管道传递树形结构或指定输入文件");
            }
            Ok(input)
        }
    }
}
