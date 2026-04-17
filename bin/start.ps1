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

$runtimeHome = Resolve-RouterAbsolutePath -BasePath (Get-Location).Path -CandidatePath $RuntimeHome
$releaseManifest = Get-RouterReleaseManifest -RuntimeHome $runtimeHome
$manifestInstallMode = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'installMode'
$manifestConfigDirectory = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'configRoot'
$manifestConfigFile = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'configFile'
$manifestDataDirectory = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'mutableDataRoot'
$manifestLogDirectory = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'logRoot'
$manifestRunDirectory = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'runRoot'
$installMode = Get-RouterNormalizedInstallMode -RequestedMode ([string]$env:SDKWORK_ROUTER_INSTALL_MODE) -FallbackMode $manifestInstallMode
$defaultConfigDirectoryRaw = Get-RouterDefaultConfigRoot -RuntimeHome $runtimeHome -InstallMode $installMode
$initialConfigDirectory = [string]$env:SDKWORK_CONFIG_DIR
if ([string]::IsNullOrWhiteSpace($initialConfigDirectory) -and -not [string]::IsNullOrWhiteSpace([string]$env:SDKWORK_CONFIG_FILE)) {
    $initialConfigDirectory = Split-Path -Parent ([string]$env:SDKWORK_CONFIG_FILE)
}
if ([string]::IsNullOrWhiteSpace($initialConfigDirectory) -and -not [string]::IsNullOrWhiteSpace($manifestConfigDirectory)) {
    $initialConfigDirectory = $manifestConfigDirectory
}
if ([string]::IsNullOrWhiteSpace($initialConfigDirectory)) {
    $initialConfigDirectory = $defaultConfigDirectoryRaw
}
$initialConfigDirectory = Resolve-RouterHostPath -PathValue $initialConfigDirectory -DefaultValue $defaultConfigDirectoryRaw
$envFile = Join-Path $initialConfigDirectory 'router.env'
Import-RouterEnvFile -EnvFile $envFile

$installMode = Get-RouterNormalizedInstallMode -RequestedMode ([string]$env:SDKWORK_ROUTER_INSTALL_MODE) -FallbackMode $manifestInstallMode
$defaultConfigDirectoryRaw = Get-RouterDefaultConfigRoot -RuntimeHome $runtimeHome -InstallMode $installMode
$defaultDataDirectoryRaw = Get-RouterDefaultDataRoot -RuntimeHome $runtimeHome -InstallMode $installMode
$defaultLogDirectoryRaw = Get-RouterDefaultLogRoot -RuntimeHome $runtimeHome -InstallMode $installMode
$defaultRunDirectoryRaw = Get-RouterDefaultRunRoot -RuntimeHome $runtimeHome -InstallMode $installMode
$defaultConfigFileRaw = Join-Path $defaultConfigDirectoryRaw 'router.yaml'

$configDirectoryRaw = [string]$env:SDKWORK_CONFIG_DIR
if ([string]::IsNullOrWhiteSpace($configDirectoryRaw) -and -not [string]::IsNullOrWhiteSpace([string]$env:SDKWORK_CONFIG_FILE)) {
    $configDirectoryRaw = Split-Path -Parent ([string]$env:SDKWORK_CONFIG_FILE)
}
if ([string]::IsNullOrWhiteSpace($configDirectoryRaw) -and -not [string]::IsNullOrWhiteSpace($manifestConfigDirectory)) {
    $configDirectoryRaw = $manifestConfigDirectory
}
if ([string]::IsNullOrWhiteSpace($configDirectoryRaw)) {
    $configDirectoryRaw = $defaultConfigDirectoryRaw
}
$configDirectory = Resolve-RouterHostPath -PathValue $configDirectoryRaw -DefaultValue $defaultConfigDirectoryRaw

$configFileRaw = [string]$env:SDKWORK_CONFIG_FILE
if ([string]::IsNullOrWhiteSpace($configFileRaw) -and -not [string]::IsNullOrWhiteSpace($manifestConfigFile)) {
    $configFileRaw = $manifestConfigFile
}
if ([string]::IsNullOrWhiteSpace($configFileRaw)) {
    $configFileRaw = $defaultConfigFileRaw
}
$configFilePath = Resolve-RouterHostPath -PathValue $configFileRaw -DefaultValue $defaultConfigFileRaw

$dataDirectoryRaw = if (-not [string]::IsNullOrWhiteSpace($manifestDataDirectory)) {
    $manifestDataDirectory
} else {
    $defaultDataDirectoryRaw
}
$dataDirectory = Resolve-RouterHostPath -PathValue $dataDirectoryRaw -DefaultValue $defaultDataDirectoryRaw

$logDirectoryRaw = if (-not [string]::IsNullOrWhiteSpace($manifestLogDirectory)) {
    $manifestLogDirectory
} else {
    $defaultLogDirectoryRaw
}
$logDirectory = Resolve-RouterHostPath -PathValue $logDirectoryRaw -DefaultValue $defaultLogDirectoryRaw

$runDirectoryRaw = if (-not [string]::IsNullOrWhiteSpace($manifestRunDirectory)) {
    $manifestRunDirectory
} else {
    $defaultRunDirectoryRaw
}
$runDirectory = Resolve-RouterHostPath -PathValue $runDirectoryRaw -DefaultValue $defaultRunDirectoryRaw

$binDir = Join-Path $runtimeHome 'bin'
$binaryPath = Join-Path $binDir $binaryName
$pidFile = Join-Path $runDirectory 'router-product-service.pid'
$stateFile = Join-Path $runDirectory 'router-product-service.state.env'
$stdoutLog = Join-Path $logDirectory 'router-product-service.stdout.log'
$stderrLog = Join-Path $logDirectory 'router-product-service.stderr.log'
$planFile = Join-Path $runDirectory 'router-product-service.plan.json'
$defaultAdminSiteDir = Join-Path $runtimeHome 'sites\admin\dist'
$defaultPortalSiteDir = Join-Path $runtimeHome 'sites\portal\dist'
$defaultBootstrapDataDir = Join-Path $runtimeHome 'data'
$repositoryBootstrapDataDir = Join-Path $repoRoot 'data'
$defaultConfigDirPortable = Convert-ToRouterPortablePath -PathValue $configDirectory
$defaultConfigFilePortable = Convert-ToRouterPortablePath -PathValue $configFilePath
$defaultDatabaseUrl = Get-RouterDefaultDatabaseUrl -DataRoot $dataDirectory -InstallMode $installMode
$defaultAdminSiteDirPortable = Convert-ToRouterPortablePath -PathValue $defaultAdminSiteDir
$defaultPortalSiteDirPortable = Convert-ToRouterPortablePath -PathValue $defaultPortalSiteDir

Ensure-RouterDirectory -DirectoryPath $configDirectory
Ensure-RouterDirectory -DirectoryPath $dataDirectory
Ensure-RouterDirectory -DirectoryPath $logDirectory
Ensure-RouterDirectory -DirectoryPath $runDirectory

if (-not $env:SDKWORK_ROUTER_BINARY) {
    $env:SDKWORK_ROUTER_BINARY = $binaryPath
}
if (-not $env:SDKWORK_CONFIG_DIR) {
    $env:SDKWORK_CONFIG_DIR = $defaultConfigDirPortable
}
if (-not $env:SDKWORK_CONFIG_FILE) {
    $env:SDKWORK_CONFIG_FILE = $defaultConfigFilePortable
}
if (-not $env:SDKWORK_DATABASE_URL) {
    $env:SDKWORK_DATABASE_URL = $defaultDatabaseUrl
}
if (-not $env:SDKWORK_ROUTER_INSTALL_MODE) {
    $env:SDKWORK_ROUTER_INSTALL_MODE = $installMode
}
if (-not $env:SDKWORK_BOOTSTRAP_PROFILE) {
    $env:SDKWORK_BOOTSTRAP_PROFILE = 'prod'
}
if (-not $env:SDKWORK_BOOTSTRAP_DATA_DIR) {
    if (Test-Path $defaultBootstrapDataDir -PathType Container) {
        $env:SDKWORK_BOOTSTRAP_DATA_DIR = Convert-ToRouterPortablePath -PathValue $defaultBootstrapDataDir
    } elseif (Test-Path $repositoryBootstrapDataDir -PathType Container) {
        $env:SDKWORK_BOOTSTRAP_DATA_DIR = Convert-ToRouterPortablePath -PathValue $repositoryBootstrapDataDir
    }
}
if (-not $env:SDKWORK_WEB_BIND) {
    $env:SDKWORK_WEB_BIND = '0.0.0.0:3001'
}
if (-not $env:SDKWORK_GATEWAY_BIND) {
    $env:SDKWORK_GATEWAY_BIND = '127.0.0.1:8080'
}
if (-not $env:SDKWORK_ADMIN_BIND) {
    $env:SDKWORK_ADMIN_BIND = '127.0.0.1:8081'
}
if (-not $env:SDKWORK_PORTAL_BIND) {
    $env:SDKWORK_PORTAL_BIND = '127.0.0.1:8082'
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
$env:SDKWORK_ROUTER_INSTALL_MODE = Get-RouterNormalizedInstallMode -RequestedMode ([string]$env:SDKWORK_ROUTER_INSTALL_MODE) -FallbackMode $installMode
$env:SDKWORK_CONFIG_DIR = Resolve-RouterHostPath -PathValue $env:SDKWORK_CONFIG_DIR -DefaultValue $defaultConfigDirPortable
$env:SDKWORK_CONFIG_FILE = Resolve-RouterHostPath -PathValue $env:SDKWORK_CONFIG_FILE -DefaultValue $defaultConfigFilePortable
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
