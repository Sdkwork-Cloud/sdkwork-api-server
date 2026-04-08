param(
    [Alias('Home')]
    [string]$RuntimeHome = '',
    [switch]$Foreground,
    [switch]$DryRun,
    [int]$WaitSeconds = 60,
    [string]$Bind = '',
    [string]$ConfigDir = '',
    [string]$ConfigFile = '',
    [string]$DatabaseUrl = '',
    [string]$Roles = '',
    [string]$NodeIdPrefix = '',
    [string]$GatewayBind = '',
    [string]$AdminBind = '',
    [string]$PortalBind = '',
    [string]$GatewayUpstream = '',
    [string]$AdminUpstream = '',
    [string]$PortalUpstream = '',
    [string]$AdminSiteDir = '',
    [string]$PortalSiteDir = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path (Split-Path -Parent $PSCommandPath) 'lib\runtime-common.ps1')

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Get-RouterRepoRoot -ScriptDirectory $scriptDir
$defaultHome = Get-RouterDefaultInstallHome -RepoRoot $repoRoot
$binaryName = Get-RouterBinaryName -BaseName 'router-product-service'

if ([string]::IsNullOrWhiteSpace($RuntimeHome)) {
    $siblingBinary = Join-Path $scriptDir $binaryName
    if (Test-Path $siblingBinary) {
        $RuntimeHome = Split-Path -Parent $scriptDir
    } else {
        $RuntimeHome = $defaultHome
    }
}

$runtimeHome = if (Test-Path $RuntimeHome) { (Resolve-Path $RuntimeHome).Path } else { $RuntimeHome }
$binDir = Join-Path $runtimeHome 'bin'
$binaryPath = Join-Path $binDir $binaryName
$configDirectory = Join-Path $runtimeHome 'config'
$varDirectory = Join-Path $runtimeHome 'var'
$dataDirectory = Join-Path $varDirectory 'data'
$logDirectory = Join-Path $varDirectory 'log'
$runDirectory = Join-Path $varDirectory 'run'
$envFile = Join-Path $configDirectory 'router.env'
$pidFile = Join-Path $runDirectory 'router-product-service.pid'
$stateFile = Join-Path $runDirectory 'router-product-service.state.env'
$stdoutLog = Join-Path $logDirectory 'router-product-service.stdout.log'
$stderrLog = Join-Path $logDirectory 'router-product-service.stderr.log'
$planFile = Join-Path $runDirectory 'router-product-service.plan.json'
$defaultAdminSiteDir = Join-Path $runtimeHome 'sites\admin\dist'
$defaultPortalSiteDir = Join-Path $runtimeHome 'sites\portal\dist'
$defaultConfigDirPortable = Convert-ToRouterPortablePath -PathValue $configDirectory
$defaultDatabaseUrl = "sqlite://$(Convert-ToRouterPortablePath -PathValue $dataDirectory)/sdkwork-api-router.db"
$defaultAdminSiteDirPortable = Convert-ToRouterPortablePath -PathValue $defaultAdminSiteDir
$defaultPortalSiteDirPortable = Convert-ToRouterPortablePath -PathValue $defaultPortalSiteDir

Ensure-RouterDirectory -DirectoryPath $configDirectory
Ensure-RouterDirectory -DirectoryPath $dataDirectory
Ensure-RouterDirectory -DirectoryPath $logDirectory
Ensure-RouterDirectory -DirectoryPath $runDirectory

Import-RouterEnvFile -EnvFile $envFile

if (-not $env:SDKWORK_ROUTER_BINARY) {
    $env:SDKWORK_ROUTER_BINARY = $binaryPath
}
if (-not $env:SDKWORK_CONFIG_DIR) {
    $env:SDKWORK_CONFIG_DIR = $defaultConfigDirPortable
}
if (-not $env:SDKWORK_DATABASE_URL) {
    $env:SDKWORK_DATABASE_URL = $defaultDatabaseUrl
}
if (-not $env:SDKWORK_WEB_BIND) {
    $env:SDKWORK_WEB_BIND = '0.0.0.0:9983'
}
if (-not $env:SDKWORK_GATEWAY_BIND) {
    $env:SDKWORK_GATEWAY_BIND = '127.0.0.1:9980'
}
if (-not $env:SDKWORK_ADMIN_BIND) {
    $env:SDKWORK_ADMIN_BIND = '127.0.0.1:9981'
}
if (-not $env:SDKWORK_PORTAL_BIND) {
    $env:SDKWORK_PORTAL_BIND = '127.0.0.1:9982'
}
if (-not $env:SDKWORK_ADMIN_SITE_DIR) {
    $env:SDKWORK_ADMIN_SITE_DIR = $defaultAdminSiteDirPortable
}
if (-not $env:SDKWORK_PORTAL_SITE_DIR) {
    $env:SDKWORK_PORTAL_SITE_DIR = $defaultPortalSiteDirPortable
}

if ($Bind) { $env:SDKWORK_WEB_BIND = $Bind }
if ($ConfigDir) { $env:SDKWORK_CONFIG_DIR = $ConfigDir }
if ($ConfigFile) { $env:SDKWORK_CONFIG_FILE = $ConfigFile }
if ($DatabaseUrl) { $env:SDKWORK_DATABASE_URL = $DatabaseUrl }
if ($Roles) { $env:SDKWORK_ROUTER_ROLES = $Roles }
if ($NodeIdPrefix) { $env:SDKWORK_ROUTER_NODE_ID_PREFIX = $NodeIdPrefix }
if ($GatewayBind) { $env:SDKWORK_GATEWAY_BIND = $GatewayBind }
if ($AdminBind) { $env:SDKWORK_ADMIN_BIND = $AdminBind }
if ($PortalBind) { $env:SDKWORK_PORTAL_BIND = $PortalBind }
if ($GatewayUpstream) { $env:SDKWORK_GATEWAY_PROXY_TARGET = $GatewayUpstream }
if ($AdminUpstream) { $env:SDKWORK_ADMIN_PROXY_TARGET = $AdminUpstream }
if ($PortalUpstream) { $env:SDKWORK_PORTAL_PROXY_TARGET = $PortalUpstream }
if ($AdminSiteDir) { $env:SDKWORK_ADMIN_SITE_DIR = $AdminSiteDir }
if ($PortalSiteDir) { $env:SDKWORK_PORTAL_SITE_DIR = $PortalSiteDir }
$env:SDKWORK_ROUTER_BINARY = Resolve-RouterHostPath -PathValue $env:SDKWORK_ROUTER_BINARY -DefaultValue $binaryPath
$env:SDKWORK_CONFIG_DIR = Resolve-RouterHostPath -PathValue $env:SDKWORK_CONFIG_DIR -DefaultValue $defaultConfigDirPortable
$env:SDKWORK_DATABASE_URL = Resolve-RouterHostDatabaseUrl -DatabaseUrl $env:SDKWORK_DATABASE_URL -DefaultValue $defaultDatabaseUrl
$env:SDKWORK_ADMIN_SITE_DIR = Resolve-RouterHostPath -PathValue $env:SDKWORK_ADMIN_SITE_DIR -DefaultValue $defaultAdminSiteDirPortable
$env:SDKWORK_PORTAL_SITE_DIR = Resolve-RouterHostPath -PathValue $env:SDKWORK_PORTAL_SITE_DIR -DefaultValue $defaultPortalSiteDirPortable

Push-Location $runtimeHome
try {
    $gatewayHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/api/v1/health'
    $adminHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/api/admin/health'
    $portalHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/api/portal/health'

    $existingPid = Get-RouterManagedProcessId -PidFile $pidFile -StateFile $stateFile
    if (($existingPid -gt 0) -and -not $DryRun) {
        $managedState = Get-RouterManagedState -StateFile $stateFile
        $activeWebBind = if ($managedState -and $managedState.WebBind) { $managedState.WebBind } else { $env:SDKWORK_WEB_BIND }
        $activeGatewayBind = if ($managedState -and $managedState.GatewayBind) { $managedState.GatewayBind } else { $env:SDKWORK_GATEWAY_BIND }
        $activeAdminBind = if ($managedState -and $managedState.AdminBind) { $managedState.AdminBind } else { $env:SDKWORK_ADMIN_BIND }
        $activePortalBind = if ($managedState -and $managedState.PortalBind) { $managedState.PortalBind } else { $env:SDKWORK_PORTAL_BIND }
        $activeGatewayHealthUrl = Resolve-RouterHealthUrl -BindAddress $activeWebBind -PathSuffix '/api/v1/health'
        $activeAdminHealthUrl = Resolve-RouterHealthUrl -BindAddress $activeWebBind -PathSuffix '/api/admin/health'
        $activePortalHealthUrl = Resolve-RouterHealthUrl -BindAddress $activeWebBind -PathSuffix '/api/portal/health'

        $ready = (Wait-RouterHealthUrl -Url $activeGatewayHealthUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid) `
            -and (Wait-RouterHealthUrl -Url $activeAdminHealthUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid) `
            -and (Wait-RouterHealthUrl -Url $activePortalHealthUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid)

        if ($ready) {
            if (($activeWebBind -ne $env:SDKWORK_WEB_BIND) -or ($activeGatewayBind -ne $env:SDKWORK_GATEWAY_BIND) -or ($activeAdminBind -ne $env:SDKWORK_ADMIN_BIND) -or ($activePortalBind -ne $env:SDKWORK_PORTAL_BIND)) {
                Write-RouterInfo "production runtime already running (pid=$($existingPid)) with active managed settings that differ from the requested launch configuration"
            } else {
                Write-RouterInfo "production runtime already running (pid=$($existingPid))"
            }
            Write-RouterStartupSummary `
                -Mode 'production release' `
                -WebBind $activeWebBind `
                -GatewayBind $activeGatewayBind `
                -AdminBind $activeAdminBind `
                -PortalBind $activePortalBind `
                -UnifiedAccessEnabled $true `
                -StdoutLog $stdoutLog `
                -StderrLog $stderrLog
            return
        }

        $existingProcess = Get-Process -Id $existingPid -ErrorAction SilentlyContinue
        if (-not $existingProcess) {
            Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
            Remove-RouterManagedStateFile -StateFile $stateFile
            Write-RouterInfo "previous production runtime pid=$existingPid exited during readiness checks; removed stale pid file and retrying startup"
        } else {
            Write-RouterInfo "production runtime pid=$existingPid is present but health checks are failing; recent logs follow"
            Show-RouterLogTail -LogFile $stdoutLog
            Show-RouterLogTail -LogFile $stderrLog
            Throw-RouterError "production runtime is already running (pid=$existingPid) but failed health checks"
        }
    }

    if ($DryRun) {
        $canRunInstalledDryRun = (Test-Path $env:SDKWORK_ROUTER_BINARY -PathType Leaf) `
            -and (Test-Path $env:SDKWORK_ADMIN_SITE_DIR -PathType Container) `
            -and (Test-Path $env:SDKWORK_PORTAL_SITE_DIR -PathType Container)

        if ($canRunInstalledDryRun) {
            $planOutput = & $env:SDKWORK_ROUTER_BINARY --dry-run --plan-format json
        } else {
            $planOutput = Get-RouterReleaseDryRunPlanJson `
                -ConfigDir $env:SDKWORK_CONFIG_DIR `
                -DatabaseUrl $env:SDKWORK_DATABASE_URL `
                -WebBind $env:SDKWORK_WEB_BIND `
                -GatewayBind $env:SDKWORK_GATEWAY_BIND `
                -AdminBind $env:SDKWORK_ADMIN_BIND `
                -PortalBind $env:SDKWORK_PORTAL_BIND `
                -ConfigFile $env:SDKWORK_CONFIG_FILE `
                -Roles $env:SDKWORK_ROUTER_ROLES `
                -NodeIdPrefix $env:SDKWORK_ROUTER_NODE_ID_PREFIX `
                -GatewayUpstream $env:SDKWORK_GATEWAY_PROXY_TARGET `
                -AdminUpstream $env:SDKWORK_ADMIN_PROXY_TARGET `
                -PortalUpstream $env:SDKWORK_PORTAL_PROXY_TARGET `
                -AdminSiteDir $env:SDKWORK_ADMIN_SITE_DIR `
                -PortalSiteDir $env:SDKWORK_PORTAL_SITE_DIR
        }

        Write-RouterUtf8File -FilePath $planFile -Content $planOutput
        Get-Content $planFile
        return
    }

    Assert-RouterBindAddressesAvailable `
        -BindAddresses @(
            $env:SDKWORK_WEB_BIND,
            $env:SDKWORK_GATEWAY_BIND,
            $env:SDKWORK_ADMIN_BIND,
            $env:SDKWORK_PORTAL_BIND
        ) `
        -ServiceLabel 'production runtime'

    Assert-RouterFileExists -Label 'router-product-service binary' -FilePath $env:SDKWORK_ROUTER_BINARY
    Assert-RouterDirectoryExists -Label 'admin site directory' -DirectoryPath $env:SDKWORK_ADMIN_SITE_DIR
    Assert-RouterDirectoryExists -Label 'portal site directory' -DirectoryPath $env:SDKWORK_PORTAL_SITE_DIR

    $planOutput = & $env:SDKWORK_ROUTER_BINARY --dry-run --plan-format json
    Write-RouterUtf8File -FilePath $planFile -Content $planOutput

    if ($Foreground) {
        & $env:SDKWORK_ROUTER_BINARY
        return
    }

    if (Test-Path $stdoutLog) { Remove-Item $stdoutLog -Force }
    if (Test-Path $stderrLog) { Remove-Item $stderrLog -Force }

    $process = Start-RouterBackgroundProcess `
        -FilePath $env:SDKWORK_ROUTER_BINARY `
        -WorkingDirectory $runtimeHome `
        -StdoutLog $stdoutLog `
        -StderrLog $stderrLog

    Write-RouterUtf8File -FilePath $pidFile -Content $process.Id
    Remove-RouterManagedStateFile -StateFile $stateFile

    $ready = (Wait-RouterHealthUrl -Url $gatewayHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id) `
        -and (Wait-RouterHealthUrl -Url $adminHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id) `
        -and (Wait-RouterHealthUrl -Url $portalHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id)

    if (-not $ready) {
        $runtimeExited = -not (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)
        Stop-RouterProcessTree -ProcessId $process.Id -WaitSeconds $WaitSeconds -Force | Out-Null
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $stateFile
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        if ($runtimeExited) {
            Throw-RouterError 'production runtime exited before health checks completed; see startup log above'
        }
        Throw-RouterError "router-product-service failed health checks on $($env:SDKWORK_WEB_BIND)"
    }

    if (-not (Confirm-RouterProcessAlive -ProcessId $process.Id -WaitSeconds 2)) {
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $stateFile
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        Throw-RouterError 'production runtime exited immediately after reporting ready; see startup log above'
    }

    $processFingerprint = Get-RouterProcessFingerprint -ProcessId $process.Id
    Write-RouterManagedStateFile -StateFile $stateFile `
        -ProcessId $process.Id `
        -ProcessFingerprint $processFingerprint `
        -Mode 'production release' `
        -WebBind $env:SDKWORK_WEB_BIND `
        -GatewayBind $env:SDKWORK_GATEWAY_BIND `
        -AdminBind $env:SDKWORK_ADMIN_BIND `
        -PortalBind $env:SDKWORK_PORTAL_BIND `
        -UnifiedAccessEnabled $true

    Write-RouterInfo "started router-product-service (pid=$($process.Id))"
    Write-RouterStartupSummary `
        -Mode 'production release' `
        -WebBind $env:SDKWORK_WEB_BIND `
        -GatewayBind $env:SDKWORK_GATEWAY_BIND `
        -AdminBind $env:SDKWORK_ADMIN_BIND `
        -PortalBind $env:SDKWORK_PORTAL_BIND `
        -UnifiedAccessEnabled $true `
        -StdoutLog $stdoutLog `
        -StderrLog $stderrLog
}
finally {
    Pop-Location
}
