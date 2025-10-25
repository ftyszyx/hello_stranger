# hello_stranger
hello_stranger

陌生人恋爱交友的社交平台

## 其它一些名字
Echo (回声)

## 功能需求

### 基本的聊天功能
1.聊天功能，支持发送文本、图片、语音、视频等消息
1.私聊
1.群聊
1.实时语音
1.实时视频

### 用户管理
1. 登录，注册，退出
1. qq和微信登录


### 好友功能
1. 添加好友
1. 删除好友
1. 好友列表
1. 好友请求


### 朋友圈功能
1. 发布朋友圈
1. 删除朋友圈
1. 朋友圈列表
1. 朋友圈请求

## 技术
1. 数据库使用postgres
1. 使用salvo作为web框架
1. 使用sea-orm作为orm
1. 使用sqlx作为数据库操作

## 一些命令

### 数据库迁移
```bash
sqlx migrate run
```


### 生成entity
```
sea-orm-cli generate entity -u "postgres://test:123456@localhost:5432/test_chat" -o "server/src/entities" --with-serde both
```