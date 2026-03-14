param(
    [string]$DatabaseUrl = "sqlite://sdkwork-api-server.db",
    [string]$AdminBind = "127.0.0.1:8081",
    [string]$GatewayBind = "127.0.0.1:8080",
    [string]$PortalBind = "127.0.0.1:8082",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Escape-PsLiteral([string]$Value) {
    return $Value.Replace("'", "''")
}

function Start-ServiceWindow {
    param(
        [string]$Title,
        [string]$PackageName
    )

    $command = @"
`$Host.UI.RawUI.WindowTitle = '$($Title)'
Set-Location -LiteralPath '$([string](Escape-PsLiteral $repoRoot.Path))'
`$env:SDKWORK_DATABASE_URL = '$([string](Escape-PsLiteral $DatabaseUrl))'
`$env:SDKWORK_ADMIN_BIND = '$([string](Escape-PsLiteral $AdminBind))'
`$env:SDKWORK_GATEWAY_BIND = '$([string](Escape-PsLiteral $GatewayBind))'
`$env:SDKWORK_PORTAL_BIND = '$([string](Escape-PsLiteral $PortalBind))'
cargo run -p $PackageName
"@

    if ($DryRun) {
        Write-Host "[start-servers] powershell -NoExit -Command <window '$Title' running cargo run -p $PackageName>"
        return
    }

    Start-Process powershell -ArgumentList @(
        "-NoExit",
        "-Command",
        $command
    ) | Out-Null
}

Write-Host "[start-servers] SDKWORK_DATABASE_URL=$DatabaseUrl"
Write-Host "[start-servers] SDKWORK_ADMIN_BIND=$AdminBind"
Write-Host "[start-servers] SDKWORK_GATEWAY_BIND=$GatewayBind"
Write-Host "[start-servers] SDKWORK_PORTAL_BIND=$PortalBind"

Start-ServiceWindow -Title "sdkwork admin-api-service" -PackageName "admin-api-service"
Start-ServiceWindow -Title "sdkwork gateway-service" -PackageName "gateway-service"
Start-ServiceWindow -Title "sdkwork portal-api-service" -PackageName "portal-api-service"
