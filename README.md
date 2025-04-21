使用 rust 编写的简易 docker，参考项目：https://github.com/lixd/mydocker

sudo docker run -d docker.m.daocloud.io/busybox sh
真正的 docker 运行上述指令也会直接退出。也许我应该先考虑清楚 -d 真正的应用场景，再去实现相应的功能。
比如要不要 dup2(pty, stdin)，setsid() 等等

尚未实现的功能
1. 搞清楚 tty 与后台进程，实现 run -d
2. 使用 busybox 之外的容器镜像测试
3. exec 命令
4. 容器网络
5. 实现 docker container prune 帮助我清理
6. 实现 daemon 管理（前台 & 后台）进程资源
