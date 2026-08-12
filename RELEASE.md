# 发布打包说明

本项目有两种发布方式：本地手动打包，或利用 GitHub Actions 自动发布。
两种方式最终都会得到一个包含 `chess_rust.exe` 的 zip 压缩包。

## 方式一：本地手动打包

1. 在项目根目录运行：

```powershell
cargo build --release
```

2. 运行打包脚本：

```powershell
.\package.ps1
```

脚本会自动读取 `Cargo.toml` 里的版本号，把 `target/release/chess_rust.exe`
连同 `README.md`、`LICENSE` 一起打包到：

```text
dist/chess-rust-<版本号>.zip
```

3. 打开 GitHub 仓库的 Releases 页面：

   https://github.com/Zaxaerith/chess-rust/releases

4. 点击 `Draft a new release`。
5. 填写版本号（例如 `v0.1.0`）、标题和更新说明。
6. 把 `dist/chess-rust-<版本号>.zip` 拖入 Assets 区域。
7. 点击 `Publish release`。

## 方式二：GitHub Actions 自动发布

仓库已包含 `.github/workflows/release.yml`。只要推送一个 `v*` 开头的 tag，
GitHub 会自动编译 release 版、打包 zip、创建 Release 并附带下载文件。

1. 确认 `Cargo.toml` 里的版本号正确，例如：

```toml
version = "0.1.0"
```

2. 在项目根目录打 tag 并推送：

```powershell
git tag v0.1.0
git push origin v0.1.0
```

3. 打开仓库的 **Actions** 页面，等待 `release` 工作流跑完。
4. 打开仓库的 **Releases** 页面，就能看到自动创建的 `v0.1.0` Release，
   里面带 `chess-rust-0.1.0.zip`。
5. 可以在 Release 页面点击 `Edit`，补充更新说明后再发布。

## 版本号约定

建议遵循语义化版本：

- `v0.1.0`：第一个可用版本
- `v0.2.0`：新增功能
- `v0.2.1`：修复 bug
- `v1.0.0`：稳定发布

## Release 说明模板

```text
## 更新内容

- 新增功能一
- 修复问题一
- 改进体验一

## 下载

- Windows：chess-rust-<版本号>.zip

## 运行

解压后直接双击 chess_rust.exe 即可，无需安装其他依赖。
```
