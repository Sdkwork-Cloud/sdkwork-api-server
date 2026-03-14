param(
    [switch]$Install,
    [switch]$Preview,
    [switch]$Tauri,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$consolePath = Join-Path $repoRoot "console"
$needsInstall = $Install -or -not (Test-Path (Join-Path $consolePath "node_modules"))

function Escape-PsLiteral([string]$Value) {
    return $Value.Replace("'", "''")
}

$steps = @()
if ($needsInstall) {
    $steps += "pnpm --dir console install"
}

if ($Preview) {
    $steps += "pnpm --dir console build"
    $steps += "pnpm --dir console preview"
} elseif ($Tauri) {
    $steps += "pnpm --dir console tauri:dev"
} else {
    $steps += "pnpm --dir console dev"
}

if ($DryRun) {
    foreach ($step in $steps) {
        Write-Host "[start-console] $step"
    }
    if ($Tauri) {
        Write-Host "[start-console] browser UI remains reachable on http://127.0.0.1:5173 while Tauri dev is running"
    }
    exit 0
}

$command = @"
Set-Location -LiteralPath '$([string](Escape-PsLiteral $repoRoot.Path))'
$($steps -join "`n")
"@

if ($Tauri) {
    Write-Host "[start-console] browser UI remains reachable on http://127.0.0.1:5173 while Tauri dev is running"
}

Start-Process powershell -ArgumentList @(
    "-NoExit",
    "-Command",
    $command
) | Out-Null
