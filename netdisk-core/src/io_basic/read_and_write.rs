use actix_web::web;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn async_write_toml<
    T: Serialize + Send + 'static,
    U: AsRef<Path> + std::marker::Send + 'static,
>(
    data: T,
    path: U,
) -> Result<(), io::Error> {
    let path_buf = path.as_ref().to_path_buf();
    let result = web::block(move || write_struct_to_toml(&data, &path_buf))
        .await
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("阻塞线程失败: {}", e)))??;

    Ok(result)
}
/// 同步写入结构体到toml文件中
pub fn write_struct_to_toml<T: Serialize, U: AsRef<Path>>(
    data: &T,
    path: U,
) -> Result<(), io::Error> {
    let toml_string = match toml::to_string_pretty(data) {
        Ok(s) => s,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("序列化到 TOML 失败: {}", e),
            ));
        }
    };
    fs::write(path, toml_string)?;
    Ok(())
}

pub async fn async_read_and_deserialize<T, U>(path: T) -> Result<U, io::Error>
where
    T: AsRef<Path>,
    U: for<'de> Deserialize<'de> + Send + 'static, // U 是目标结构体
{
    let path_ref = path.as_ref(); // 获取 Path 引用
    let path_display = path_ref.display().to_string(); // 提前克隆路径的字符串表示

    let mut file = File::open(path_ref)
        .await
        .map_err(|e| io::Error::new(e.kind(), format!("打开文件失败 {}: {}", path_display, e)))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .await
        .map_err(|e| io::Error::new(e.kind(), format!("读取文件内容失败: {}", e)))?;

    let result = web::block(move || {
        toml::from_str::<U>(&contents).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("TOML反序列化失败: {}", e), // 更新错误信息
            )
        })
    })
    .await;

    // 3. 处理阻塞任务和反序列化结果
    match result {
        Ok(inner_result) => inner_result,
        Err(e) => Err(io::Error::new(
            io::ErrorKind::Interrupted,
            format!("阻塞任务失败: {}", e),
        )),
    }
}
