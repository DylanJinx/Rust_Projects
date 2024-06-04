# Rust_Projects

Here are some Rust projects
在 Rust 项目上传到 GitHub 的过程中，你需要完成以下几个步骤：

1. **创建 GitHub 仓库**：

   - 登录到你的 GitHub 账户。
   - 点击右上角的“+”，选择“New repository”。
   - 填写仓库名称，选择是否公开或私有，然后点击“Create repository”。

2. **在本地项目文件夹中初始化 Git**：

   - 打开终端或命令提示符，导航到你的项目文件夹。
   - 运行以下命令初始化 Git 仓库：
     ```bash
     git init
     ```

3. **添加远程仓库地址**：

   - 使用 GitHub 提供的仓库 URL（在仓库页面可以找到），将其添加为远程仓库：
     ```bash
     git remote add origin 你的仓库URL
     ```

4. **添加文件到 Git**：

   - 将所有文件添加到本地 Git 仓库中：
     ```bash
     git add .
     ```
   - 或者你可以选择只添加特定文件：
     ```bash
     git add 文件名
     ```

5. **提交更改**：

   - 对所做的更改进行提交，并添加提交信息：
     ```bash
     git commit -m "Initial commit"
     ```

6. **推送到 GitHub**：
   - 将本地仓库的更改推送到 GitHub：
     ```bash
     git push -u origin master
     ```
   - 注意：如果你使用的是 main 分支（GitHub 新仓库的默认分支），则需要将`master`改为`main`：
     ```bash
     git push -u origin main
     ```

# 实际使用

在 Rust_Projects 目录下进行:
git add .
git status
git commit -m "命名"
git push
