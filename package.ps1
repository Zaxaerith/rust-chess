$ErrorActionPreference = "Stop"

cargo build --release
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$version = "0.1.0"
$verMatch = Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"'
if ($verMatch) {
    $version = $verMatch.Matches[0].Groups[1].Value
}

$dist = "dist/rust-chess-$version"
$zip = "dist/rust-chess-$version.zip"

Remove-Item -Recurse -Force $dist -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Copy-Item "target/release/chess_rust.exe" "$dist/rust-chess.exe"
Copy-Item "README.md" "$dist/README.md" -ErrorAction SilentlyContinue
Copy-Item "LICENSE" "$dist/LICENSE" -ErrorAction SilentlyContinue

Compress-Archive -Path "$dist/*" -DestinationPath $zip -Force

Write-Host "打包完成：$zip" -ForegroundColor Green
