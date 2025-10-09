#

## 请求接口

```fish
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"123", "client_secret":"123"}' http://127.0.0.1:8080/access_token
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"'$NETDISK_CLIENT_ID'", "client_secret":"'$NETDISK_CLIENT_SECRET'"}' http://127.0.0.1:8080/access_token
# 测试获取文件信息
curl --location 'https://open-api.123pan.com/api/v2/file/list?parentFileId=0&limit=100' \
    --header 'Content-Type: application/json' \
    --header 'Platform: open_platform' \
    --header "Authorization: Bearer $ACCESS_TOKEN"
```

## TODO
- ` /upload/v2/file/create`:创建文件
- `/upload/v2/file/slice`:上传分片
- `/upload/v2/file/upload_complete`:上传完毕
- `/upload/v2/file/domain`:获取上传域名
- `/upload/v2/file/single/create`:单步上传

# 
