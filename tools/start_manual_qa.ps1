param(
    [Parameter(Mandatory)][ValidatePattern('^[a-zA-Z0-9_-]{1,40}$')][string]$Session,
    [ValidateSet('ABC','D','EWhite','EBlack','EDraw','EAsync')][string]$Case = 'ABC',
    [switch]$Launch
)
$ErrorActionPreference = 'Stop'
$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$caseRoot = Join-Path $projectRoot "target/v020-manual/$Session/$Case"
$appRoot = Join-Path $caseRoot 'app'
$dataRoot = Join-Path $caseRoot 'data'
$sourceExe = Join-Path $projectRoot 'target/release/chess_rust.exe'
$testExe = Join-Path $appRoot 'rust-chess-qa.exe'
New-Item -ItemType Directory -Path $appRoot,$dataRoot -Force | Out-Null
if (Test-Path -LiteralPath $testExe) {
    if ((Get-FileHash -LiteralPath $testExe).Hash -ne (Get-FileHash -LiteralPath $sourceExe).Hash) {
        throw 'Existing test executable differs; choose a new Session. No files overwritten.'
    }
} else {
    Copy-Item -LiteralPath $sourceExe -Destination $testExe
}
Write-Host "Isolated settings: $appRoot/settings.cfg"
Write-Host "Isolated autosave: $dataRoot/autosave-v1.json"
Write-Host "PGN exports and notes: $caseRoot"
Write-Host "EXE SHA-256: $((Get-FileHash -LiteralPath $testExe).Hash)"
if ($Launch) {
    $existing = @(Get-Process -Name 'rust-chess-qa' -ErrorAction SilentlyContinue)
    if ($existing.Count -gt 0) { throw 'Close other QA game processes before launching.' }
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $testExe
    $info.WorkingDirectory = $appRoot
    $info.UseShellExecute = $false
    $info.Environment['RUST_CHESS_DATA_DIR'] = $dataRoot
    $process = [Diagnostics.Process]::Start($info)
    Write-Host "Started QA process $($process.Id). All window inputs remain manual."
} else {
    Write-Host 'Prepared only. Add -Launch when you choose to begin manual testing.'
}
