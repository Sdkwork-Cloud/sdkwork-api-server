[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Foreground,
    [switch]$DryRun,
    [int]$WaitSeconds = 600,
    [switch]$Install,
    [switch]$Browser,
    [switch]$Preview,
    [switch]$ProxyDev,
    [switch]$Tauri,
    [string]$DatabaseUrl = '',
    [string]$GatewayBind = '',
    [string]$AdminBind = '',
    [string]$PortalBind = '',
    [string]$WebBind = '',
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$RemainingArgs = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path (Split-Path -Parent $PSCommandPath) 'lib\runtime-common.ps1')

function Read-RouterRemainingOptionValue {
    param(
        [Parameter(Mandatory = $true)][string[]]$Args,
        [Parameter(Mandatory = $true)][ref]$Index,
        [Parameter(Mandatory = $true)][string]$Arg,
        [Parameter(Mandatory = $true)][string]$OptionName,
        [string]$InlineValue = $null
    )

    if ($null -ne $InlineValue) {
        return $InlineValue
    }

    if (($Index.Value + 1) -ge $Args.Count) {
        Throw-RouterError "$OptionName requires a value"
    }

    $Index.Value += 1
    return $Args[$Index.Value]
}

function Get-RouterManagedDevCargoTargetDir {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$DevHome
    )

    return (Get-RouterManagedDevCargoTargetDirCandidates -RepoRoot $RepoRoot -DevHome $DevHome)[0]
}

function Get-RouterManagedDevCargoTargetDirCandidates {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$DevHome
    )

    $repoRootFullPath = [System.IO.Path]::GetFullPath($RepoRoot)
    $defaultTargetDir = Join-Path $repoRootFullPath 'bin\.sdkwork-target-vs2022'
    $normalizedDevHome = [System.IO.Path]::GetFullPath($DevHome)
    $managedDefaultDevHome = [System.IO.Path]::GetFullPath(
        (Join-Path $repoRootFullPath (Join-Path 'artifacts\runtime\dev' (Get-RouterRuntimePlatformKey)))
    )
    if ([string]::Equals($normalizedDevHome, $managedDefaultDevHome, [System.StringComparison]::OrdinalIgnoreCase)) {
        return @($defaultTargetDir)
    }

    $hashAlgorithm = [System.Security.Cryptography.MD5]::Create()
    try {
        $hashBytes = $hashAlgorithm.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($normalizedDevHome))
    } finally {
        $hashAlgorithm.Dispose()
    }

    $hashSuffix = -join ($hashBytes | Select-Object -First 6 | ForEach-Object { $_.ToString('x2') })
    $candidateTargetDirs = @()

    $requestedManagedCargoRoot = [string]$env:SDKWORK_ROUTER_MANAGED_CARGO_ROOT
    if (-not [string]::IsNullOrWhiteSpace($requestedManagedCargoRoot)) {
        if ([System.IO.Path]::IsPathRooted($requestedManagedCargoRoot)) {
            $resolvedManagedCargoRoot = [System.IO.Path]::GetFullPath($requestedManagedCargoRoot)
        } else {
            $resolvedManagedCargoRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRootFullPath $requestedManagedCargoRoot))
        }

        $candidateTargetDirs += (Join-Path $resolvedManagedCargoRoot $hashSuffix)
    } else {
        $repoVolumeRoot = [System.IO.Path]::GetPathRoot($repoRootFullPath)
        if (-not [string]::IsNullOrWhiteSpace($repoVolumeRoot)) {
            $candidateTargetDirs += (Join-Path (Join-Path $repoVolumeRoot 'sdkrt') $hashSuffix)
        }
    }

    $repoLocalFallbackTargetDir = Join-Path $repoRootFullPath "bin\.sdkrt-$hashSuffix"
    if ($repoLocalFallbackTargetDir -notin $candidateTargetDirs) {
        $candidateTargetDirs += $repoLocalFallbackTargetDir
    }

    return $candidateTargetDirs
}

function Test-RouterManagedCargoTargetDirAvailable {
    param([Parameter(Mandatory = $true)][string]$TargetDir)

    $debugDirectory = Join-Path $TargetDir 'debug'
    $lockFile = Join-Path $debugDirectory '.cargo-lock'
    $lockHandle = $null

    try {
        Ensure-RouterDirectory -DirectoryPath $debugDirectory
        $lockHandle = [System.IO.File]::Open(
            $lockFile,
            [System.IO.FileMode]::OpenOrCreate,
            [System.IO.FileAccess]::ReadWrite,
            [System.IO.FileShare]::None
        )
        return $true
    } catch {
        return $false
    } finally {
        if ($null -ne $lockHandle) {
            $lockHandle.Dispose()
        }
    }
}

function Get-RouterAvailableManagedDevCargoTargetDir {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$DevHome
    )

    foreach ($preferredTargetDir in (Get-RouterManagedDevCargoTargetDirCandidates -RepoRoot $RepoRoot -DevHome $DevHome)) {
        if (Test-RouterManagedCargoTargetDirAvailable -TargetDir $preferredTargetDir) {
            return $preferredTargetDir
        }

        $targetParent = Split-Path -Parent $preferredTargetDir
        $targetLeaf = Split-Path -Leaf $preferredTargetDir
        for ($attempt = 1; $attempt -le 16; $attempt++) {
            $candidateTargetDir = Join-Path $targetParent "$targetLeaf-r$attempt"
            if (Test-RouterManagedCargoTargetDirAvailable -TargetDir $candidateTargetDir) {
                return $candidateTargetDir
            }
        }
    }

    Throw-RouterError "failed to reserve a writable managed cargo target directory for development workspace $DevHome"
}

if ($RemainingArgs.Count -gt 0) {
    for ($index = 0; $index -lt $RemainingArgs.Count; $index++) {
        $arg = $RemainingArgs[$index]
        $optionName = $arg
        $inlineValue = $null

        if ($arg.StartsWith('--') -and $arg.Contains('=')) {
            $parts = $arg.Split('=', 2)
            $optionName = $parts[0]
            $inlineValue = $parts[1]
        }

        switch ($optionName) {
            '--foreground' {
                if ($null -ne $inlineValue) { Throw-RouterError "$optionName does not accept a value" }
                $Foreground = $true
                break
            }
            '--dry-run' {
                if ($null -ne $inlineValue) { Throw-RouterError "$optionName does not accept a value" }
                $DryRun = $true
                break
            }
            '--install' {
                if ($null -ne $inlineValue) { Throw-RouterError "$optionName does not accept a value" }
                $Install = $true
                break
            }
            '--browser' {
                if ($null -ne $inlineValue) { Throw-RouterError "$optionName does not accept a value" }
                $Browser = $true
                break
            }
            '--preview' {
                if ($null -ne $inlineValue) { Throw-RouterError "$optionName does not accept a value" }
                $Preview = $true
                break
            }
            '--proxy-dev' {
                if ($null -ne $inlineValue) { Throw-RouterError "$optionName does not accept a value" }
                $ProxyDev = $true
                break
            }
            '--tauri' {
                if ($null -ne $inlineValue) { Throw-RouterError "$optionName does not accept a value" }
                $Tauri = $true
                break
            }
            '--wait-seconds' {
                $waitSecondsValue = Read-RouterRemainingOptionValue -Args $RemainingArgs -Index ([ref]$index) -Arg $arg -OptionName $optionName -InlineValue $inlineValue
                try {
                    $WaitSeconds = [int]$waitSecondsValue
                } catch {
                    Throw-RouterError "invalid value for --wait-seconds: $waitSecondsValue"
                }
                break
            }
            '--database-url' {
                $DatabaseUrl = Read-RouterRemainingOptionValue -Args $RemainingArgs -Index ([ref]$index) -Arg $arg -OptionName $optionName -InlineValue $inlineValue
                break
            }
            '--gateway-bind' {
                $GatewayBind = Read-RouterRemainingOptionValue -Args $RemainingArgs -Index ([ref]$index) -Arg $arg -OptionName $optionName -InlineValue $inlineValue
                break
            }
            '--admin-bind' {
                $AdminBind = Read-RouterRemainingOptionValue -Args $RemainingArgs -Index ([ref]$index) -Arg $arg -OptionName $optionName -InlineValue $inlineValue
                break
            }
            '--portal-bind' {
                $PortalBind = Read-RouterRemainingOptionValue -Args $RemainingArgs -Index ([ref]$index) -Arg $arg -OptionName $optionName -InlineValue $inlineValue
                break
            }
            '--web-bind' {
                $WebBind = Read-RouterRemainingOptionValue -Args $RemainingArgs -Index ([ref]$index) -Arg $arg -OptionName $optionName -InlineValue $inlineValue
                break
            }
            default {
                Throw-RouterError "unknown option: $arg"
            }
        }
    }
}

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

$packagedBootstrapDataDirectory = Join-Path $scriptDir 'data'
$repositoryBootstrapDataDirectory = Join-Path $repoRoot 'data'
if (-not $env:SDKWORK_DATABASE_URL) {
    $env:SDKWORK_DATABASE_URL = "sqlite://$(Convert-ToRouterPortablePath -PathValue $dataDirectory)/sdkwork-api-router-dev.db"
}
if (-not $env:SDKWORK_BOOTSTRAP_PROFILE) {
    $env:SDKWORK_BOOTSTRAP_PROFILE = 'dev'
}
if (-not $env:SDKWORK_BOOTSTRAP_DATA_DIR) {
    if (Test-Path $repositoryBootstrapDataDirectory -PathType Container) {
        $env:SDKWORK_BOOTSTRAP_DATA_DIR = Convert-ToRouterPortablePath -PathValue $repositoryBootstrapDataDirectory
    } elseif (Test-Path $packagedBootstrapDataDirectory -PathType Container) {
        $env:SDKWORK_BOOTSTRAP_DATA_DIR = Convert-ToRouterPortablePath -PathValue $packagedBootstrapDataDirectory
    }
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
    $ProxyDev = $false
    $Tauri = $false
} elseif ($Tauri) {
    $Preview = $false
    $ProxyDev = $false
} elseif ($ProxyDev) {
    $Preview = $false
    $Tauri = $false
} elseif (-not $Preview) {
    $Preview = $true
}

$workspaceLauncher = Join-Path $repoRoot 'scripts\dev\start-workspace.mjs'
Assert-RouterFileExists -Label 'workspace launcher' -FilePath $workspaceLauncher

$adminNodeModules = Join-Path $repoRoot 'apps\sdkwork-router-admin\node_modules'
$portalNodeModules = Join-Path $repoRoot 'apps\sdkwork-router-portal\node_modules'
$adminNodeModulesHealthy = Test-RouterPnpmNodeModulesHealthy -NodeModulesPath $adminNodeModules
$portalNodeModulesHealthy = Test-RouterPnpmNodeModulesHealthy -NodeModulesPath $portalNodeModules
$portalFrontendLinksUsable = Test-RouterPortalFrontendDependencyLinksUsable -RepoRoot $repoRoot

if ((-not $portalNodeModulesHealthy) -and (-not $portalFrontendLinksUsable)) {
    Write-RouterInfo 'portal frontend node_modules are missing or were transplanted from another workspace; attempting local repair'
    if (Repair-RouterPortalFrontendNodeModules -RepoRoot $repoRoot) {
        $portalFrontendLinksUsable = Test-RouterPortalFrontendDependencyLinksUsable -RepoRoot $repoRoot
        if ($portalFrontendLinksUsable) {
            Write-RouterInfo 'portal frontend dependency links repaired from local workspace packages'
        }
    }
}

if (-not $adminNodeModulesHealthy) {
    Write-RouterInfo 'admin frontend node_modules are missing or stale; forcing --install'
}
if ((-not $portalNodeModulesHealthy) -and (-not $portalFrontendLinksUsable)) {
    Write-RouterInfo 'portal frontend dependency links are still incomplete after local repair; forcing --install'
}

if ($Install -or -not $adminNodeModulesHealthy -or ((-not $portalNodeModulesHealthy) -and (-not $portalFrontendLinksUsable))) {
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
if ($ProxyDev) { $startArgs += '--proxy-dev' }
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

    if ($Preview -or $Tauri -or $ProxyDev) {
        $primaryAdminSurfaceUrl = $previewAdminUrl
        $primaryPortalSurfaceUrl = $previewPortalUrl
        $primaryMode = if ($Tauri) { 'development tauri' } elseif ($ProxyDev) { 'development proxy hot reload' } else { 'development preview' }
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
                Throw-RouterError "development workspace already running (pid=$($existingPid)) with active managed settings that differ from the requested launch configuration; run bin/stop-dev.ps1 before relaunching with different settings"
            }
            Write-RouterInfo "development workspace already running (pid=$($existingPid))"
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

    if ((Test-RouterWindowsPlatform) -and [string]::IsNullOrWhiteSpace([string]$env:CARGO_TARGET_DIR)) {
        $preferredManagedCargoTargetDir = Get-RouterManagedDevCargoTargetDir -RepoRoot $repoRoot -DevHome $devHome
        $selectedManagedCargoTargetDir = Get-RouterAvailableManagedDevCargoTargetDir -RepoRoot $repoRoot -DevHome $devHome
        if (-not [string]::Equals(
            $preferredManagedCargoTargetDir,
            $selectedManagedCargoTargetDir,
            [System.StringComparison]::OrdinalIgnoreCase
        )) {
            Write-RouterInfo "managed cargo target dir is unavailable or busy; using fallback $selectedManagedCargoTargetDir"
        }
        $env:CARGO_TARGET_DIR = $selectedManagedCargoTargetDir
    }

    $cargoEnv = Enable-RouterManagedCargoEnv -RepoRoot $repoRoot
    if ($cargoEnv.Enabled) {
        $env:SDKWORK_ROUTER_USE_PREBUILT_BACKEND_BINARIES = '1'
        $env:SDKWORK_ROUTER_USE_PREBUILT_WEB_BINARY = '1'
        Write-RouterInfo "CARGO_TARGET_DIR=$($cargoEnv.CargoTargetDir)"
        if (-not [string]::IsNullOrWhiteSpace($cargoEnv.CargoBuildJobs)) {
            Write-RouterInfo "CARGO_BUILD_JOBS=$($cargoEnv.CargoBuildJobs)"
        }
        if ($DryRun) {
            Write-RouterInfo "backend warm-up: $(Get-RouterWindowsBackendWarmupCommandDisplay -CargoBuildJobs $cargoEnv.CargoBuildJobs)"
        }
    } else {
        Remove-Item Env:SDKWORK_ROUTER_USE_PREBUILT_BACKEND_BINARIES -ErrorAction SilentlyContinue
        Remove-Item Env:SDKWORK_ROUTER_USE_PREBUILT_WEB_BINARY -ErrorAction SilentlyContinue
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
    } elseif ($ProxyDev) {
        $preflightBindAddresses += @(
            $env:SDKWORK_WEB_BIND,
            '127.0.0.1:5173',
            '127.0.0.1:5174'
        )
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
