#

## 请求接口

```fish
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"123", "client_secret":"123"}' http://127.0.0.1:8080/access_token
curl -X POST -H 'Content-Type: application/json' -d '{"client_id":"'$NETDISK_CLIENT_ID'", "client_secret":"'$NETDISK_CLIENT_SECRET'"}' http://127.0.0.1:8080/access_token
```
