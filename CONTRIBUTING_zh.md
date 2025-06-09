# 贡献指南

感谢您考虑为 Yjs 协作编辑服务器贡献代码！

1. 将本仓库 Fork 到您的 GitHub 账号。
2. 克隆您的 Fork 并添加上游仓库：
   ```bash
   git clone https://github.com/Wenrh2004/yjs-collaboration-server.git
   cd yjs-collaboration-server
   git remote add upstream https://github.com/Wenrh2004/yjs-collaboration-server.git
   ```
3. 创建功能分支：
   ```bash
   git checkout -b feature/amazing-feature
   ```
4. 提交您的修改：
   ```bash
   git add .
   git commit -m "feat: 添加了很棒的功能"
   ```
5. 与上游仓库同步并 rebase：
   ```bash
   git fetch upstream
   git rebase upstream/master
   ```
6. 将分支推送到您的仓库并发起 Pull Request，请在 PR 描述中说明您的改动。 