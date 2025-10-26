#

## 请求接口

```fish
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"123", "client_secret":"123"}' http://127.0.0.1:8080/access_token
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"'$NETDISK_CLIENT_ID'", "client_secret":"'$NETDISK_CLIENT_SECRET'"}' http://127.0.0.1:8080/access_token
# 测试获取文件信息
curl --location 'http://127.0.0.1:8080/file/file_lists_query?parentFileId=0&limit=100'
# 获得单个文件信息
curl --location 'http://127.0.0.1:8080/file/file_query?fileID=18226271'
# 获取文件详细
curl -X POST -H 'Content-Type: application/json' -d '{"fileIds":[18226271]}' http://127.0.0.1:8080/file/files_info
# 创建文件
curl -X POST -H 'Content-Type: application/json' -d '{"name":"path1","parentID":0}' http://127.0.0.1:8080/file/mkdir
# 移动文件到特定目录
curl -X POST 'http://127.0.0.1:8080/file/move' -H 'Content-Type: application/json' -d '{"fileIDs": [18999095],"toParentFileID": 18529409}'

# 文件移动到垃圾桶
curl -X POST -H "Content-Type: application/json" -d '{"fileIds": [18226271]}' http://127.0.0.1:8080/trash

# 删除文件
curl -X POST -H "Content-Type: application/json" -d '{"fileIds": [18226271]}' http://127.0.0.1:8080/delete

# 获取分享文件列表
curl --location 'http://127.0.0.1:8080/share/list?limit=10&lastShareId=0

# 创建文件分享链接
curl -X POST -H "Content-Type: application/json" -d '{
                                                        "shareName": "测试分享链接",
                                                        "shareExpire": "1",
                                                        "fileIDList": "18869763"
                                                    }'  http://127.0.0.1:8080/share/create
# 上传文件
 curl -X POST -H 'Content-Type: application/json' -d  '{
                                                        "parentFileID": 0,
                                                        "filename": "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv",                                                                             "etag": "e325c611ea19f1bc3bef16f0eac7cb92",
                                                        "size": 59570941009
                                                    }' http://127.0.0.1:8080/file/upload
# 获取文件下载信息
 curl -X GET -H 'Content-Type: application/json'  http://127.0.0.1:8080/file/download?fileId=18340536

# 获取付费链接列表
 curl --location 'http://127.0.0.1:8080/share/payment/list?limit=10&lastShareId=0'

# 设置共享链接参数
curl -X PUT -H "Content-Type: application/json" -d '{
                                                       "shareIdList": [69692575],
                                                       "trafficSwitch": 2,
                                                       "trafficLimitSwitch": 2,
                                                       "trafficLimit": 1073741824
                                                   }'  http://127.0.0.1:8080/share/list/info

# 付费链接
curl -X PUT -H "Content-Type: application/json" -d '{
                              "shareName": "测试付费分享链接",
                              "fileIDList": "11522388,11522389",
                              "isReward": 0,
                              "payAmount": 10,
                              "resourceDesc": "这是我的测试付费分享链接，用来测试openapi"
                          }'  http://127.0.0.1:8080/share/content-payment/creat

# 修改付费分享链接
curl -X PUT -H "Content-Type: application/json" -d '{
                                    "shareIdList": [69692575],
                                    "trafficSwitch": 2,
                                    "trafficLimitSwitch": 2,
                                    "trafficLimit": 1073741824}'  http://127.0.0.1:8080/share/list/payment/info
```

```bash
# 获取文件的详细信息
curl --location 'https://open-api.123pan.com/api/v1/file/detail?fileID=18226271' \
                                    --header 'Content-Type: application/json' \
                                    --header 'Platform: open_platform' \
                                    --header "Authorization: Bearer $ACCESS_TOKEN"
# 获取用户信息
curl -X GET -H 'Content-Type: application/json'  http://127.0.0.1:8080/user_info


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

- `etag`:
<!-- ||||| -->
