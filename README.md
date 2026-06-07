生成迁移文件
```
cargo run --bin cli -- migration generate
```
迁移
```
cargo run --bin cli -- migration apply
```

//TODO 
- 关键字过滤
- 标记统计
- webui
  - 筛选
  - 管理管理员界面
- bot
  - 权限指令只能私聊触发
- 日志系统

//FIXME
举报的接口需要数据库里有人
但是举报不应该在ban列表里
待解决