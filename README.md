生成迁移文件
```
cargo run --bin cli -- migration generate
```
迁移
```
cargo run --bin cli -- migration apply
```

# TODO 
- [ ] 关键字过滤
- [x] 标记统计
- [x] webui
  - [x] 筛选
  - [x] 管理管理员界面
- [x] bot
  - [x] 权限指令只能私聊触发
- [ ] 日志系统
- [ ] 配置

