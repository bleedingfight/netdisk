#[cfg(test)]
mod tests {
    use actix_web::dev::ServiceFactory;
    use actix_web::dev::ServiceRequest;
    use actix_web::web;
    use actix_web::{test, web::Bytes, App};
    use netdisk_core::netdisk_api::base_api::*;
    use std::error::Error;
    pub fn create_app() -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response = actix_web::body::BoxBody,
            Error = Error,
            InitError = (),
        >,
    > {
        // 您的路由和服务定义
        App::new().service(web::resource("/file_query").route(web::get().to(file_query)))
    }
    // 使用 actix_web::test::actix_web_test 宏来创建一个测试，它会设置 Actix Web 运行时环境
    // 也可以使用 tokio::test，但 Actix Web 推荐使用自己的 test 宏
    #[actix_web::test]
    async fn test_file_query_success() {
        // 1. 设置应用
        let app = test::init_service(create_app()).await;

        // 2. 构建请求
        // 相当于 GET /file_query?file_id=18226271
        let req = test::TestRequest::get()
            .uri("/file_query?file_id=18226271")
            .to_request();

        // 3. 发送请求并获取响应
        let resp = test::call_service(&app, req).await;

        // 4. 断言验证
        // 检查状态码
        assert!(resp.status().is_success()); // 200 OK

        // 检查响应体内容
        let response_body = test::read_body(resp).await;
        // 期望响应体是 "Found file with ID: 18226271"
        assert_eq!(
            response_body,
            Bytes::from_static(b"Found file with ID: 18226271")
        );
    }

    #[actix_web::test]
    async fn test_file_query_not_found() {
        // 1. 设置应用
        let app = test::init_service(create_app()).await;

        // 2. 构建请求，使用一个不存在的 file_id
        // 相当于 GET /file_query?file_id=999
        let req = test::TestRequest::get()
            .uri("/file_query?file_id=999")
            .to_request();

        // 3. 发送请求并获取响应
        let resp = test::call_service(&app, req).await;

        // 4. 断言验证
        // 检查状态码：期望是 404 Not Found
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);

        // 检查响应体内容
        let response_body = test::read_body(resp).await;
        // 期望响应体是 "File ID 999 not found"
        assert_eq!(response_body, Bytes::from_static(b"File ID 999 not found"));
    }
}

