param(
    [switch]$Execute
)

$ErrorActionPreference = 'Stop'

$root = 'C:\Users\admin\.codex\memories'
$rootFull = [System.IO.Path]::GetFullPath($root).TrimEnd('\') + '\'

$patterns = @(
    '^\.pnpm-store$',
    '^m2repo$',
    '^cargo-target-',
    '^cargo-temp$',
    '^claw-studio-cargo-target$',
    '^claw-studio-rust-target$',
    '^magic-studio-cargo-target$',
    '^magic-studio-vite-cache$',
    '^npm-cache$',
    '^openclaw-prepare-cache$',
    '^openclaw-runtime-cache$',
    '^sdkwork-target$',
    '^sdkwork-windows-mirror$',
    '^target-step04',
    '^target-step05',
    '^tmp$'
)

if (-not (Test-Path -LiteralPath $root)) {
    throw "Directory not found: $root"
}

$targets = Get-ChildItem -LiteralPath $root -Force -Directory | Where-Object {
    $name = $_.Name
    foreach ($pattern in $patterns) {
        if ($name -match $pattern) {
            return $true
        }
    }
    return $false
} | Sort-Object Name

if ($targets.Count -eq 0) {
    Write-Host 'No matching non-memory directories found.'
    exit 0
}

Write-Host "Root: $root"
Write-Host "Matched directories: $($targets.Count)"
$targets | Select-Object Name, FullName | Format-Table -AutoSize

if (-not $Execute) {
    Write-Host ''
    Write-Host 'Dry run only. Re-run with -Execute to delete these directories.'
    exit 0
}

$deleted = New-Object System.Collections.Generic.List[string]
$failed = New-Object System.Collections.Generic.List[string]

foreach ($target in $targets) {
    try {
        $full = [System.IO.Path]::GetFullPath($target.FullName)
        if (-not $full.StartsWith($rootFull, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to delete outside root: $full"
        }

        if (Test-Path -LiteralPath $full) {
            Remove-Item -LiteralPath $full -Recurse -Force -ErrorAction Stop
            $deleted.Add($target.Name) | Out-Null
        }
    }
    catch {
        $failed.Add("$($target.Name) :: $($_.Exception.Message)") | Out-Null
    }
}

Write-Host ''
Write-Host "Deleted: $($deleted.Count)"
Write-Host "Failed: $($failed.Count)"

if ($failed.Count -gt 0) {
    Write-Host ''
    Write-Host 'Failures:'
    $failed
    exit 1
}
