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


```
