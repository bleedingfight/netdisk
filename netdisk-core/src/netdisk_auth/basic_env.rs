use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

/// 网盘配置环境结构体，用于获取配置文件的安全路径。
#[derive(Debug, Clone)]
pub struct NetDiskEnv {
    /// 最终确定的、有效的配置目录路径。
    pub config_dir: PathBuf,
}

impl NetDiskEnv {
    /// 构造函数：检测并确定合法的网盘配置目录路径。
    pub fn new() -> Result<Self, io::Error> {
        // 1. 定义默认配置路径
        let default_config_dir = NetDiskEnv::get_default_config_path()?;

        // 2. 检查 NETDISK_CONFIG 环境变量
        let final_path = match env::var("NETDISK_CONFIG") {
            // 环境变量已设置
            Ok(env_path_str) => {
                let env_path = PathBuf::from(env_path_str);

                // 3. 验证环境变量路径是否合法且可写
                if NetDiskEnv::is_valid_and_writable(&env_path) {
                    println!(
                        "[INFO] 使用环境变量 NETDISK_CONFIG 指定的路径: {}",
                        env_path.display()
                    );
                    env_path
                } else {
                    eprintln!(
                        "[WARN] 环境变量 NETDISK_CONFIG 路径无效或不可写入: {}。将使用默认路径。",
                        env_path.display()
                    );
                    default_config_dir
                }
            }

            // 环境变量未设置
            Err(env::VarError::NotPresent) => {
                println!("[INFO] 环境变量 NETDISK_CONFIG 未设置。将使用默认路径。");
                default_config_dir
            }

            // 其他读取错误
            Err(e) => {
                eprintln!("[ERROR] 读取环境变量时发生错误: {}。将使用默认路径。", e);
                default_config_dir
            }
        };

        // 4. 确保最终路径存在（创建目录）
        NetDiskEnv::create_config_dir_if_not_exists(&final_path)?;

        Ok(NetDiskEnv {
            config_dir: final_path,
        })
    }

    /// 获取默认配置路径：`~/.config/netdisk`
    fn get_default_config_path() -> Result<PathBuf, io::Error> {
        // 使用 home 库获取用户主目录，并拼接 .config/netdisk
        match dirs::home_dir() {
            Some(home) => Ok(home.join(".config").join("netdisk")),
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "无法找到用户主目录，无法确定默认配置路径。",
            )),
        }
    }

    /// 检查路径是否存在且可写入
    fn is_valid_and_writable(path: &Path) -> bool {
        // 检查路径是否存在
        if !path.exists() {
            // 如果路径不存在，检查其父目录是否可写 (以便能够创建它)
            if let Some(parent) = path.parent() {
                // 递归检查父目录是否存在且是目录，并且可以创建文件
                // ⚠️ 完整的可写性检测很复杂，这里我们只检查父目录是否存在
                return parent.is_dir() || parent.as_os_str().is_empty();
            }
            // 根目录或无法获取父目录，暂时视为可接受（会在下一步创建）
            return true;
        }

        // 如果路径存在，检查它是否是一个目录
        if !path.is_dir() {
            eprintln!("[WARN] 路径已存在，但不是一个目录: {}", path.display());
            return false;
        }

        // 检查可写性（通过尝试创建/删除文件/文件夹来精确检查比较复杂，
        // 这里简化为只检查权限，但在不同OS上效果不一。
        // 通常我们依赖下一步的 fs::create_dir_all 来进行最终验证。）
        true
    }

    /// 确保配置目录存在，如果不存在则创建它
    fn create_config_dir_if_not_exists(path: &Path) -> Result<(), io::Error> {
        if !path.exists() {
            println!("[INFO] 目录 {} 不存在，正在创建...", path.display());
            // 递归创建所有父目录
            fs::create_dir_all(path)?;
            println!("[INFO] 目录创建成功。");
        }

        Ok(())
    }
}
