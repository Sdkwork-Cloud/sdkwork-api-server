param(
    [switch]$Foreground,
    [switch]$DryRun,
    [int]$WaitSeconds = 600,
    [switch]$Install,
    [switch]$Browser,
    [switch]$Preview,
    [switch]$Tauri,
    [string]$DatabaseUrl = '',
    [string]$GatewayBind = '',
    [string]$AdminBind = '',
    [string]$PortalBind = '',
    [string]$WebBind = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path (Split-Path -Parent $PSCommandPath) 'lib\runtime-common.ps1')

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Get-RouterRepoRoot -ScriptDirectory $scriptDir
$devHome = Get-RouterDefaultDevHome -RepoRoot $repoRoot
$configDirectory = Join-Path $devHome 'config'
$dataDirectory = Join-Path $devHome 'data'
$logDirectory = Join-Path $devHome 'log'
$runDirectory = Join-Path $devHome 'run'
$envFile = Join-Path $configDirectory 'router-dev.env'
$pidFile = Join-Path $runDirectory 'start-workspace.pid'
$stopFile = Join-Path $runDirectory 'start-workspace.stop'
$stateFile = Join-Path $runDirectory 'start-workspace.state.env'
$stdoutLog = Join-Path $logDirectory 'start-workspace.stdout.log'
$stderrLog = Join-Path $logDirectory 'start-workspace.stderr.log'
$planFile = Join-Path $runDirectory 'start-workspace.plan.txt'

Ensure-RouterDirectory -DirectoryPath $configDirectory
Ensure-RouterDirectory -DirectoryPath $dataDirectory
Ensure-RouterDirectory -DirectoryPath $logDirectory
Ensure-RouterDirectory -DirectoryPath $runDirectory

Import-RouterEnvFile -EnvFile $envFile

if (-not $env:SDKWORK_DATABASE_URL) {
    $env:SDKWORK_DATABASE_URL = "sqlite://$(Convert-ToRouterPortablePath -PathValue $dataDirectory)/sdkwork-api-router-dev.db"
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
if (-not $env:SDKWORK_WEB_BIND) {
    $env:SDKWORK_WEB_BIND = '127.0.0.1:9983'
}

if ($DatabaseUrl) { $env:SDKWORK_DATABASE_URL = $DatabaseUrl }
if ($GatewayBind) { $env:SDKWORK_GATEWAY_BIND = $GatewayBind }
if ($AdminBind) { $env:SDKWORK_ADMIN_BIND = $AdminBind }
if ($PortalBind) { $env:SDKWORK_PORTAL_BIND = $PortalBind }
if ($WebBind) { $env:SDKWORK_WEB_BIND = $WebBind }

if ($Browser) {
    $Preview = $false
    $Tauri = $false
} elseif ($Tauri) {
    $Preview = $false
} elseif (-not $Preview) {
    $Preview = $true
}

$workspaceLauncher = Join-Path $repoRoot 'scripts\dev\start-workspace.mjs'
Assert-RouterFileExists -Label 'workspace launcher' -FilePath $workspaceLauncher

$adminNodeModules = Join-Path $repoRoot 'apps\sdkwork-router-admin\node_modules'
$portalNodeModules = Join-Path $repoRoot 'apps\sdkwork-router-portal\node_modules'
if ($Install -or -not (Test-Path $adminNodeModules) -or -not (Test-Path $portalNodeModules)) {
    $Install = $true
}

$startArgs = @(
    'scripts/dev/start-workspace.mjs',
    '--database-url', $env:SDKWORK_DATABASE_URL,
    '--gateway-bind', $env:SDKWORK_GATEWAY_BIND,
    '--admin-bind', $env:SDKWORK_ADMIN_BIND,
    '--portal-bind', $env:SDKWORK_PORTAL_BIND,
    '--web-bind', $env:SDKWORK_WEB_BIND,
    '--stop-file', $stopFile
)

if ($Install) { $startArgs += '--install' }
if ($Preview) { $startArgs += '--preview' }
if ($Tauri) { $startArgs += '--tauri' }

$planArgs = @($startArgs + '--dry-run')

Push-Location $repoRoot
try {
    $gatewayHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_GATEWAY_BIND -PathSuffix '/health'
    $adminHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_ADMIN_BIND -PathSuffix '/admin/health'
    $portalHealthUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_PORTAL_BIND -PathSuffix '/portal/health'
    $previewAdminUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/admin/'
    $previewPortalUrl = Resolve-RouterHealthUrl -BindAddress $env:SDKWORK_WEB_BIND -PathSuffix '/portal/'
    $browserAdminUrl = 'http://127.0.0.1:5173/admin/'
    $browserPortalUrl = 'http://127.0.0.1:5174/portal/'

    if ($Preview -or $Tauri) {
        $primaryAdminSurfaceUrl = $previewAdminUrl
        $primaryPortalSurfaceUrl = $previewPortalUrl
        $primaryMode = if ($Tauri) { 'development tauri' } else { 'development preview' }
        $primaryUnifiedAccessEnabled = $true
        $secondaryAdminSurfaceUrl = $browserAdminUrl
        $secondaryPortalSurfaceUrl = $browserPortalUrl
        $secondaryMode = 'development browser'
        $secondaryUnifiedAccessEnabled = $false
    } else {
        $primaryAdminSurfaceUrl = $browserAdminUrl
        $primaryPortalSurfaceUrl = $browserPortalUrl
        $primaryMode = 'development browser'
        $primaryUnifiedAccessEnabled = $false
        $secondaryAdminSurfaceUrl = $previewAdminUrl
        $secondaryPortalSurfaceUrl = $previewPortalUrl
        $secondaryMode = 'development preview'
        $secondaryUnifiedAccessEnabled = $true
    }

    $existingPid = Get-RouterManagedProcessId -PidFile $pidFile -StateFile $stateFile
    if (($existingPid -gt 0) -and -not $DryRun) {
        $managedState = Get-RouterManagedState -StateFile $stateFile
        $activeWebBind = if ($managedState -and $managedState.WebBind) { $managedState.WebBind } else { $env:SDKWORK_WEB_BIND }
        $activeGatewayBind = if ($managedState -and $managedState.GatewayBind) { $managedState.GatewayBind } else { $env:SDKWORK_GATEWAY_BIND }
        $activeAdminBind = if ($managedState -and $managedState.AdminBind) { $managedState.AdminBind } else { $env:SDKWORK_ADMIN_BIND }
        $activePortalBind = if ($managedState -and $managedState.PortalBind) { $managedState.PortalBind } else { $env:SDKWORK_PORTAL_BIND }
        $activeGatewayHealthUrl = Resolve-RouterHealthUrl -BindAddress $activeGatewayBind -PathSuffix '/health'
        $activeAdminHealthUrl = Resolve-RouterHealthUrl -BindAddress $activeAdminBind -PathSuffix '/admin/health'
        $activePortalHealthUrl = Resolve-RouterHealthUrl -BindAddress $activePortalBind -PathSuffix '/portal/health'

        $backendReady = (Wait-RouterHealthUrl -Url $activeGatewayHealthUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid) `
            -and (Wait-RouterHealthUrl -Url $activeAdminHealthUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid) `
            -and (Wait-RouterHealthUrl -Url $activePortalHealthUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid)

        $adminSurfaceUrl = if ($managedState -and $managedState.AdminAppUrl) { $managedState.AdminAppUrl } else { $primaryAdminSurfaceUrl }
        $portalSurfaceUrl = if ($managedState -and $managedState.PortalAppUrl) { $managedState.PortalAppUrl } else { $primaryPortalSurfaceUrl }
        $mode = if ($managedState -and $managedState.Mode) { $managedState.Mode } else { $primaryMode }
        $unifiedAccessEnabled = if ($managedState) { [bool]$managedState.UnifiedAccessEnabled } else { $primaryUnifiedAccessEnabled }
        $webReady = $false

        if ($backendReady) {
            $webReady = (Wait-RouterHealthUrl -Url $adminSurfaceUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid) `
                -and (Wait-RouterHealthUrl -Url $portalSurfaceUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid)

            if ((-not $webReady) -and (-not $managedState)) {
                $adminSurfaceUrl = $secondaryAdminSurfaceUrl
                $portalSurfaceUrl = $secondaryPortalSurfaceUrl
                $mode = $secondaryMode
                $unifiedAccessEnabled = $secondaryUnifiedAccessEnabled
                $webReady = (Wait-RouterHealthUrl -Url $adminSurfaceUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid) `
                    -and (Wait-RouterHealthUrl -Url $portalSurfaceUrl -WaitSeconds $WaitSeconds -ProcessId $existingPid)
            }
        }

        if ($backendReady -and $webReady) {
            $requestedConfigurationDiffers = ($activeWebBind -ne $env:SDKWORK_WEB_BIND) `
                -or ($activeGatewayBind -ne $env:SDKWORK_GATEWAY_BIND) `
                -or ($activeAdminBind -ne $env:SDKWORK_ADMIN_BIND) `
                -or ($activePortalBind -ne $env:SDKWORK_PORTAL_BIND) `
                -or ($mode -ne $primaryMode)

            if ($requestedConfigurationDiffers) {
                Write-RouterInfo "development workspace already running (pid=$($existingPid)) with active managed settings that differ from the requested launch configuration"
            } else {
                Write-RouterInfo "development workspace already running (pid=$($existingPid))"
            }
            Write-RouterStartupSummary `
                -Mode $mode `
                -WebBind $activeWebBind `
                -GatewayBind $activeGatewayBind `
                -AdminBind $activeAdminBind `
                -PortalBind $activePortalBind `
                -UnifiedAccessEnabled $unifiedAccessEnabled `
                -AdminAppUrl $adminSurfaceUrl `
                -PortalAppUrl $portalSurfaceUrl `
                -StdoutLog $stdoutLog `
                -StderrLog $stderrLog
            return
        }

        $existingProcess = Get-Process -Id $existingPid -ErrorAction SilentlyContinue
        if (-not $existingProcess) {
            Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
            Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
            Remove-RouterManagedStateFile -StateFile $stateFile
            Write-RouterInfo "previous development workspace pid=$existingPid exited during readiness checks; removed stale pid file and retrying startup"
        } else {
            Write-RouterInfo "development workspace pid=$existingPid is present but managed services are not healthy; recent logs follow"
            Show-RouterLogTail -LogFile $stdoutLog
            Show-RouterLogTail -LogFile $stderrLog
            Throw-RouterError "development workspace is already running (pid=$existingPid) but failed health checks"
        }
    }

    $cargoEnv = Enable-RouterManagedCargoEnv -RepoRoot $repoRoot
    if ($cargoEnv.Enabled) {
        Write-RouterInfo "CARGO_TARGET_DIR=$($cargoEnv.CargoTargetDir)"
        if (-not [string]::IsNullOrWhiteSpace($cargoEnv.CargoBuildJobs)) {
            Write-RouterInfo "CARGO_BUILD_JOBS=$($cargoEnv.CargoBuildJobs)"
        }
        if ($DryRun) {
            Write-RouterInfo "backend warm-up: $(Get-RouterWindowsBackendWarmupCommandDisplay -CargoBuildJobs $cargoEnv.CargoBuildJobs)"
        }
    }

    $planOutput = & node @planArgs
    Write-RouterUtf8File -FilePath $planFile -Content $planOutput

    if ($DryRun) {
        Get-Content $planFile
        return
    }

    $preflightBindAddresses = @(
        $env:SDKWORK_GATEWAY_BIND,
        $env:SDKWORK_ADMIN_BIND,
        $env:SDKWORK_PORTAL_BIND
    )
    if ($Preview -or $Tauri) {
        $preflightBindAddresses += $env:SDKWORK_WEB_BIND
    } else {
        $preflightBindAddresses += @('127.0.0.1:5173', '127.0.0.1:5174')
    }

    Assert-RouterBindAddressesAvailable `
        -BindAddresses $preflightBindAddresses `
        -ServiceLabel 'development workspace'

    if ($cargoEnv.Enabled) {
        Invoke-RouterWindowsBackendWarmupBuild -RepoRoot $repoRoot -CargoBuildJobs $cargoEnv.CargoBuildJobs
    }

    if ($Foreground) {
        if (Test-Path $stopFile) { Remove-Item $stopFile -Force -ErrorAction SilentlyContinue }
        & node @startArgs
        return
    }

    if (Test-Path $stopFile) { Remove-Item $stopFile -Force -ErrorAction SilentlyContinue }
    if (Test-Path $stdoutLog) { Remove-Item $stdoutLog -Force }
    if (Test-Path $stderrLog) { Remove-Item $stderrLog -Force }

    $process = Start-RouterBackgroundProcess `
        -FilePath 'node' `
        -ArgumentList $startArgs `
        -WorkingDirectory $repoRoot `
        -StdoutLog $stdoutLog `
        -StderrLog $stderrLog

    Write-RouterUtf8File -FilePath $pidFile -Content $process.Id
    Remove-RouterManagedStateFile -StateFile $stateFile

    $backendReady = (Wait-RouterHealthUrl -Url $gatewayHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id) `
        -and (Wait-RouterHealthUrl -Url $adminHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id) `
        -and (Wait-RouterHealthUrl -Url $portalHealthUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id)

    if (-not $backendReady) {
        $workspaceExited = -not (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)
        Stop-RouterProcessTree -ProcessId $process.Id -WaitSeconds $WaitSeconds -Force | Out-Null
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $stateFile
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        if ($workspaceExited) {
            Throw-RouterError 'development workspace exited before backend health checks completed; see startup log above'
        }
        Throw-RouterError 'development services failed health checks'
    }

    $adminSurfaceUrl = $primaryAdminSurfaceUrl
    $portalSurfaceUrl = $primaryPortalSurfaceUrl

    $webReady = (Wait-RouterHealthUrl -Url $adminSurfaceUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id) `
        -and (Wait-RouterHealthUrl -Url $portalSurfaceUrl -WaitSeconds $WaitSeconds -ProcessId $process.Id)

    if (-not $webReady) {
        $workspaceExited = -not (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)
        Stop-RouterProcessTree -ProcessId $process.Id -WaitSeconds $WaitSeconds -Force | Out-Null
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $stateFile
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        if ($workspaceExited) {
            Throw-RouterError 'development workspace exited before web surfaces became ready; see startup log above'
        }
        Throw-RouterError 'development web surfaces failed health checks'
    }

    if (-not (Confirm-RouterProcessAlive -ProcessId $process.Id -WaitSeconds 2)) {
        Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
        Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $stateFile
        Show-RouterLogTail -LogFile $stdoutLog
        Show-RouterLogTail -LogFile $stderrLog
        Throw-RouterError 'development workspace exited immediately after reporting ready; see startup log above'
    }

    $mode = $primaryMode
    $processFingerprint = Get-RouterProcessFingerprint -ProcessId $process.Id
    Write-RouterManagedStateFile -StateFile $stateFile `
        -ProcessId $process.Id `
        -ProcessFingerprint $processFingerprint `
        -Mode $mode `
        -WebBind $env:SDKWORK_WEB_BIND `
        -GatewayBind $env:SDKWORK_GATEWAY_BIND `
        -AdminBind $env:SDKWORK_ADMIN_BIND `
        -PortalBind $env:SDKWORK_PORTAL_BIND `
        -UnifiedAccessEnabled $primaryUnifiedAccessEnabled `
        -AdminAppUrl $adminSurfaceUrl `
        -PortalAppUrl $portalSurfaceUrl

    Write-RouterInfo "started development workspace (pid=$($process.Id))"

    Write-RouterStartupSummary `
        -Mode $mode `
        -WebBind $env:SDKWORK_WEB_BIND `
        -GatewayBind $env:SDKWORK_GATEWAY_BIND `
        -AdminBind $env:SDKWORK_ADMIN_BIND `
        -PortalBind $env:SDKWORK_PORTAL_BIND `
        -UnifiedAccessEnabled $primaryUnifiedAccessEnabled `
        -AdminAppUrl $adminSurfaceUrl `
        -PortalAppUrl $portalSurfaceUrl `
        -StdoutLog $stdoutLog `
        -StderrLog $stderrLog
}
finally {
    Pop-Location
}
