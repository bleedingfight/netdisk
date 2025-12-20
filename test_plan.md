# File Query接口单元测试计划

## 接口分析
- **接口名称**: file_query
- **HTTP方法**: GET
- **URL**: `/api/v1/file/detail`
- **功能**: 查询单个文件的详细信息
- **参数**: `FileQuery`结构体，包含`file_id`字段（序列化为`fileID`）
- **返回**: `FileResponse`类型，包含文件详细信息

## 测试用例设计

### 1. 正常查询测试 (test_file_query_success)
- **目的**: 验证正常查询文件详情的功能
- **输入**: 有效的file_id和访问令牌
- **预期**: 返回成功的HTTP响应，包含文件详细信息
- **验证点**: 
  - HTTP状态码为200
  - 响应体可解析为FileResponse
  - file_id和filename字段有效

### 2. 参数验证测试 (test_file_query_invalid_params)
- **目的**: 验证接口对无效参数的处理
- **输入**: 无效的文件ID（如负数）
- **预期**: 返回适当的错误响应
- **验证点**: 参数验证逻辑正确执行

### 3. 认证失败测试 (test_file_query_auth_failure)
- **目的**: 验证接口对无效访问令牌的处理
- **输入**: 无效的访问令牌
- **预期**: 返回401未授权错误
- **验证点**: 认证逻辑正确执行

### 4. 路由集成测试 (test_file_query_route_integration)
- **目的**: 验证路由配置和请求处理
- **输入**: 通过HTTP路由发送查询请求
- **预期**: 路由正确处理请求参数
- **验证点**: 路由配置正确，参数传递无误

### 5. 序列化测试 (test_file_query_serialization)
- **目的**: 验证FileQuery参数的序列化
- **输入**: FileQuery结构体实例
- **预期**: 正确序列化为JSON，字段名正确
- **验证点**: file_id字段正确序列化为fileID

### 6. 响应结构测试 (test_file_response_structure)
- **目的**: 验证FileResponse响应结构的解析
- **输入**: 模拟的JSON响应数据
- **预期**: 正确解析为FileResponse结构体
- **验证点**: 所有必需字段正确解析

## 测试文件结构
```
netdisk-core/tests/test_file_query_api.rs
```
包含所有上述测试用例的完整实现。

## 依赖项
- actix-web: Web框架和测试工具
- serde_json: JSON序列化/反序列化
- chrono: 时间处理
- tempfile: 临时文件处理（如需要）

## 运行命令
```bash
cd netdisk-core
cargo test test_file_query
```

## 注意事项
1. 由于file_query接口依赖外部API，测试中可能需要mock外部调用
2. 测试用例应处理网络错误和API返回的各种状态码
3. 确保测试环境的访问令牌配置正确
4. 测试应覆盖正常和异常情况