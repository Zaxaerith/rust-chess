param(
    [string]$OutputRoot = "dist",
    [switch]$SkipBuild,
    [switch]$VerifyOnly,
    [switch]$AllowUnresolvedLicenseForLocalQA
)
$ErrorActionPreference = "Stop"
$projectRoot = [System.IO.Path]::GetFullPath($PSScriptRoot)
Set-Location -LiteralPath $projectRoot
if (-not $SkipBuild) {
    cargo build --release --locked -j 2
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
python tools/collect_licenses.py
if ($LASTEXITCODE -ne 0) { throw "Dependency license collection failed" }
python -B -m unittest discover -s tools -p test_collect_licenses.py
if ($LASTEXITCODE -ne 0) { throw "Dependency license regression gate failed" }
$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
$packageOutput = [System.IO.Path]::GetFullPath((Join-Path $projectRoot $OutputRoot))
if (-not $packageOutput.StartsWith($projectRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) { throw "Output must be inside this project" }
$licenseIssues = @(Get-Content -Raw 'LICENSES/dependencies/manifest.json' | ConvertFrom-Json | Where-Object { $_.note -match 'MANUAL LICENSE REVIEW REQUIRED' })
if ($licenseIssues.Count -gt 0) {
    $qaRoot = [System.IO.Path]::GetFullPath((Join-Path $projectRoot 'target')) + [System.IO.Path]::DirectorySeparatorChar
    if (-not $AllowUnresolvedLicenseForLocalQA -or -not $packageOutput.StartsWith($qaRoot,[System.StringComparison]::OrdinalIgnoreCase)) { throw 'Unresolved upstream license review; public packaging blocked. QA exception is limited to target/.' }
}
if ($VerifyOnly) {
    Write-Host "License checks completed: $($licenseIssues.Count) unresolved; no package generated."
    return
}
$packageDir = Join-Path $packageOutput "rust-chess-$version"
$packageZip = Join-Path $packageOutput "rust-chess-$version.zip"
if ((Test-Path -LiteralPath $packageDir) -or (Test-Path -LiteralPath $packageZip)) { throw "Package destination already exists; choose another OutputRoot. Existing releases are preserved." }
New-Item -ItemType Directory -Path $packageDir -Force | Out-Null
Copy-Item -LiteralPath "target/release/chess_rust.exe" -Destination (Join-Path $packageDir "rust-chess.exe")
# The repository acceptance report records the ZIP hash after packaging; keep it
# outside the archive to avoid a self-referential checksum.
foreach ($name in @('README.md','README_ZH.md','CHANGELOG.md','LICENSE','THIRD_PARTY_NOTICES.md')) { Copy-Item -LiteralPath $name -Destination (Join-Path $packageDir $name) }
Copy-Item -LiteralPath "LICENSES" -Destination (Join-Path $packageDir "LICENSES") -Recurse
New-Item -ItemType Directory -Path (Join-Path $packageDir 'assets') | Out-Null
Copy-Item -LiteralPath 'assets/ATTRIBUTION.md' -Destination (Join-Path $packageDir 'assets/ATTRIBUTION.md')
if ($licenseIssues.Count -gt 0) { [System.IO.File]::WriteAllText((Join-Path $packageDir 'QA_ONLY.txt'), 'NOT FOR PUBLIC DISTRIBUTION: upstream Pleco 0.5.0 LICENSE-MIT review remains unresolved.', [System.Text.UTF8Encoding]::new($false)) }
$checksums = Get-ChildItem -LiteralPath $packageDir -Recurse -File | Sort-Object FullName | ForEach-Object { '{0}  {1}' -f (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), [System.IO.Path]::GetRelativePath($packageDir,$_.FullName).Replace('\','/') }
[System.IO.File]::WriteAllLines((Join-Path $packageDir 'checksums.sha256'),$checksums,[System.Text.UTF8Encoding]::new($false))
Compress-Archive -Path (Join-Path $packageDir '*') -DestinationPath $packageZip
if ($licenseIssues.Count -eq 0) {
    python -B tools/verify_package.py $packageZip
    if ($LASTEXITCODE -ne 0) { throw "Formal package integrity/whitelist verification failed" }
}
Write-Host "Local package: $packageZip"
