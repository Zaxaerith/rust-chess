# 上传到 GitHub 的辅助脚本
#
# 使用步骤：
# 1. 打开 https://github.com/new 创建一个空仓库（不要勾选 README/LICENSE/.gitignore）
# 2. 复制仓库地址，例如 https://github.com/你的用户名/chess-rust.git
# 3. 在本项目根目录运行 PowerShell，然后执行：
#    Set-ExecutionPolicy -Scope Process Bypass
#    .\github_upload.ps1

$ErrorActionPreference = "Stop"

$repoUrl = Read-Host "粘贴你的 GitHub 仓库地址"
if ([string]::IsNullOrWhiteSpace($repoUrl)) {
    Write-Error "仓库地址不能为空"
    exit 1
}

if (-not (git config user.name)) {
    $name = Read-Host "GitHub 用户名（用于提交记录）"
    git config user.name $name
}
if (-not (git config user.email)) {
    $email = Read-Host "GitHub 邮箱（用于提交记录）"
    git config user.email $email
}

git branch -M main
git remote remove origin 2>$null
git remote add origin $repoUrl
git add -A
git commit -m "Initial commit: Rust 本地国际象棋"
git push -u origin main

Write-Host "上传完成：$repoUrl" -ForegroundColor Green
