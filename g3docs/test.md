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
# shared
curl --location 'https://open-api.123pan.com/api/v1/share/create' \
                              --header 'Content-Type: application/json' \
                              --header 'Platform: open_platform' \
                              --header "Authorization: Bearer $ACCESS_TOKEN" \
                              --data '{
                              "shareName": "测试分享链接",
                              "shareExpire": 1,
                              "fileIDList": "18869763"
                          }'
# 获取分享文件列表
 curl --location 'https://open-api.123pan.com/api/v1/share/list?limit=10&lastShareId=0' \
                              --header 'Content-Type: application/json' \
                              --header 'Platform: open_platform' \
                              --header "Authorization: Bearer $ACCESS_TOKEN"
# 修改分享文件信息
curl --location --request PUT 'https://open-api.123pan.com/api/v1/share/list/info' \
                              --header 'Content-Type: application/json' \
                              --header 'Platform: open_platform' \
                              --header "Authorization: Bearer $ACCESS_TOKEN" \
                              --data '{
                              "shareIdList": [69692575],
                              "trafficSwitch": 2,
                              "trafficLimitSwitch": 2,
                              "trafficLimit": 1073741824
                          }'
# 修改付费分享链接
curl --location 'https://open-api.123pan.com/api/v1/share/content-payment/create' \
                              --header 'Content-Type: application/json' \
                              --header 'Platform: open_platform' \
                              --header "Authorization: Bearer $ACCESS_TOKEN" \
                              --data '{
                              "shareName": "测试付费分享链接",
                              "fileIDList": "69692574",
                              "isReward": 0,
                              "payAmount": 10,
                              "resourceDesc": "这是我的测试付费分享链接，用来测试openapi"
                          }'

# 获取文件下载地址
curl --location 'https://open-api.123pan.com/api/v1/file/download_info?fileId=18340533' \
                              --header 'Content-Type: application/json' \
                              --header 'Platform: open_platform' \
                              --header "Authorization: Bearer $ACCESS_TOKEN"
# 上传文件
curl --location 'https://open-api.123pan.com/upload/v2/file/create' \
                              --header 'Content-Type: application/json' \
                              --header 'Platform: open_platform' \
                              --header "Authorization: Bearer $ACCESS_TOKEN" \
                              --data '{
                              "parentFileID": 0,
                              "filename": "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv",
                              "etag": "e325c611ea19f1bc3bef16f0eac7cb92",
                              "size": 59570941009
                          }'

```
