pub mod limit_deserializer {
    use serde::{Deserialize, Deserializer};

    // 定义最大限制常量
    const MAX_LIMIT: usize = 100;

    // 核心：泛型反序列化函数，用于处理 Vec<T> 类型
    // T 必须是可反序列化的，我们只关注 Vec 的长度
    pub fn limit_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        // 尝试反序列化完整的 Vec
        let mut vec_data = Vec::<T>::deserialize(deserializer)?;

        // 检查长度并截断
        if vec_data.len() > MAX_LIMIT {
            // 注意：这里我们使用 .truncate() 进行切片
            vec_data.truncate(MAX_LIMIT);
            // 可以在此处添加日志警告
            // log::warn!("List was automatically truncated to {} elements.", MAX_LIMIT);
        }

        Ok(vec_data)
    }
}
