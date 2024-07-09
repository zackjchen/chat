# Geektime Rust 语言训练营
第五周

## 创建migrate
```shell
# 1.安装sqlx-cli
cargo install sqlx-cli --no-default-features  --features rustls --features postgres

# 2.pgcli创建一个database, chat
create database chat
# 命令行 dropdb chat
# 命令行 createdb chat
# sqlx migration run

# 3.创建一个, 里面会生成一个<timestamp>_<initial>.sql脚本
sqlx migrate add initial

# 4.sql脚本写好后

# 5.运行migrate， 需要database url
# 5.1 创建一个.env文件写入DATABASE_URL=xxx 或者设置参数--database-url
sqlx migrate run

# 6. 连接到database查看tables
pgcli chat

# 7.\dt 列出所有tables，可以查看到_sqlx_migrations


```
