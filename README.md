#

## 请求接口

```fish
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"123", "client_secret":"123"}' http://127.0.0.1:8080/access_token
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"'$NETDISK_CLIENT_ID'", "client_secret":"'$NETDISK_CLIENT_SECRET'"}' http://127.0.0.1:8080/access_token
# 测试获取文件信息
curl --location 'http://127.0.0.1:8080/file_lists_query?parentFileId=0&limit=100'
# 获得单个文件信息
curl --location 'http://127.0.0.1:8080/file_query?fileID=18226271'
# 获取文件详细
curl -X POST -H 'Content-Type: application/json' -d '{"fileIds":[18226271]}' http://127.0.0.1:8080/files_info
# 创建文件
curl -X POST -H 'Content-Type: application/json' -d '{"name":"path1","parentID":0}' http://127.0.0.1:8080/mkdir
# 文件移动到垃圾桶
curl -X POST -H "Content-Type: application/json" -d '{"fileIds": [18226271]}' http://127.0.0.1:8080/trash

# 删除文件
curl -X POST -H "Content-Type: application/json" -d '{"fileIds": [18226271]}' http://127.0.0.1:8080/delete

```

```bash
# 获取文件的详细信息
curl --location 'https://open-api.123pan.com/api/v1/file/detail?fileID=18226271' \
                                    --header 'Content-Type: application/json' \
                                    --header 'Platform: open_platform' \
                                    --header "Authorization: Bearer $ACCESS_TOKEN"

```

## TODO

### 文件管理

|接口名称|接口地址|功能|实现完成|
|:---:|:-----:|:-----:|:-----:|
|上传|`/upload/v2/file/create`|创建文件|Y|
|上传|`/upload/v2/file/slice`|上传分片|否|
|上传|`/upload/v2/file/upload_complete`|上传完毕|否|
|上传|`/upload/v2/file/domain`|获取上传域名|否|
|上传|`/upload/v2/file/single/create`|单步上传|否|
|重命名|`/api/v1/file/name`|修改文件名称|否|
|重命名|`/api/v1/file/rename`|批量修改文件名称|否|
|删除|`/api/v1/file/trash`|将文件移动到垃圾桶|否|
|删除|`/api/v1/file/recover`|从回收站恢复文件|否|
|删除|`/api/v1/file/delete`|彻底删除文件|否|
|文件详情|`/api/v1/file/detail`|获取单个文件的详情|Y|
|文件详情|`/api/v1/file/infos`|获取多个文件的详情|Y|
|文件列表|`/api/v2/file/list`|获取文件列表|Y|
|移动|`/api/v1/file/move`|批量移动文件（最多100个）|否|
|下载|`/api/v1/file/download_info`|获取文件的下载地址|否|
<!-- ||||| -->
