## 接口测试

```bash
# 查询文件列表
curl -X POST --location 'https://open-api.123pan.com/api/v1/file/infos' \
                           --header 'Content-Type: application/json' \ 
                           --header 'Platform: open_platform' \
                           --header "Authorization: Bearer $ACCESS_TOKEN" \
                           -d '{"fileIds": [18226271]}'

# 创建文件
curl --location 'https://open-api.123pan.com/upload/v1/file/mkdir' \
                        --header 'Content-Type: application/json' \
                        --header 'Platform: open_platform' \
                        --header "Authorization: Bearer $ACCESS_TOKEN" \
                        --data '{
                        "name": "test_dir",
                        "parentID": 0
                    }'

# 删除文件
curl --location 'https://open-api.123pan.com/api/v1/file/trash' \
                         --header 'Content-Type: application/json' \
                         --header 'Platform: open_platform' \
                         --header "Authorization: Bearer $ACCESS_TOKEN" \
                         --data '{
                          "fileIDs": [
                             14705301,
                             14705306
                         ]
                     }'

# 移动文件
curl --location 'https://open-api.123pan.com/api/v1/file/rename' \
                       --header 'Content-Type: application/json' \
                       --header 'Platform: open_platform' \
                       --header "Authorization: Bearer $ACCESS_TOKEN" \
                       --data '{
                       "fileIDs": [
                           18779583
                       ],
                       "toParentFileID": 0
                   }'

# 获取用户信息
curl --location 'https://open-api.123pan.com/api/v1/user/info' \
                        --header 'Content-Type: application/json' \
                        --header 'Platform: open_platform' \
                        --header "Authorization: Bearer $ACCESS_TOKEN"

# 获取文件列表
curl --location 'https://open-api.123pan.com/api/v2/file/list?parentFileId=0&limit=100' \
                        --header 'Content-Type: application/json' \
                        --header 'Platform: open_platform' \
                        --header "Authorization: Bearer $ACCESS_TOKEN"
# 下载文件信息
 curl --location 'https://open-api.123pan.com/api/v1/file/download_info?fileId=18340536' \
                                               --header 'Content-Type: application/json' \
                                               --header 'Platform: open_platform' \
                                               --header "Authorization: Bearer $ACCESS_TOKEN"

```
