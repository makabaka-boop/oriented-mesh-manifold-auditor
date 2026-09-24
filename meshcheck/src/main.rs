//! meshcheck 命令行入口。
//!
//! 用法：
//!   meshcheck <file.mesh>   审计文件
//!   meshcheck -             从标准输入读取
//!   meshcheck               无参数时也从标准输入读取
//!
//! 退出码：0 审计通过；1 拓扑不封闭（含见证）；2 输入被拒绝或用法错误。

use std::io::Read;
use std::process::ExitCode;

use meshcheck::{audit, parse, Audit, EdgeKind, Finding};

const USAGE: &str = "usage: meshcheck [FILE]
  FILE  mesh description file, or - for standard input";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return ExitCode::from(0);
    }
    if args.len() > 1 {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }

    let input = match args.first().map(String::as_str) {
        None | Some("-") => {
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("error: failed to read stdin: {e}");
                return ExitCode::from(2);
            }
            buf
        }
        Some(path) => match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {path}: {e}");
                return ExitCode::from(2);
            }
        },
    };

    let mesh = match parse(&input) {
        Ok(mesh) => mesh,
        Err(e) => {
            eprintln!("REJECT {e}");
            return ExitCode::from(2);
        }
    };

    match audit(&mesh) {
        Audit::Ok {
            vertices,
            edges,
            faces,
            euler,
            genus,
        } => {
            println!(
                "OK V={vertices} E={edges} F={faces} euler={euler} genus={genus}",
            );
            ExitCode::from(0)
        }
        Audit::Failed(finding) => {
            match finding {
                Finding::Edge(a) => {
                    let kind = match a.kind {
                        EdgeKind::Boundary => "boundary",
                        EdgeKind::Orientation => "orientation",
                        EdgeKind::NonManifold => "non-manifold",
                    };
                    println!("FAIL edge {kind} {}-{} witness=edge({},{})", a.lo, a.hi, a.lo, a.hi);
                }
                Finding::Vertex { id, sectors } => {
                    println!("FAIL vertex non-manifold {id} sectors={sectors} witness=vertex({id})");
                }
                Finding::Connectivity { face_index } => {
                    println!("FAIL connectivity face_index={face_index} witness=face#{face_index}");
                }
            }
            ExitCode::from(1)
        }
    }
}
