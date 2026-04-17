param(
    [Alias('Home')]
    [string]$RuntimeHome = '',
    [switch]$DryRun,
    [int]$WaitSeconds = 30,
    [switch]$GracefulOnly
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
$manifestLogDirectory = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'logRoot'
$manifestRunDirectory = Get-RouterReleaseManifestString -Manifest $releaseManifest -PropertyName 'runRoot'
$installMode = Get-RouterNormalizedInstallMode -RequestedMode ([string]$env:SDKWORK_ROUTER_INSTALL_MODE) -FallbackMode $manifestInstallMode
$defaultConfigDirectoryRaw = Get-RouterDefaultConfigRoot -RuntimeHome $runtimeHome -InstallMode $installMode
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
$envFile = Join-Path $configDirectory 'router.env'
Import-RouterEnvFile -EnvFile $envFile

$installMode = Get-RouterNormalizedInstallMode -RequestedMode ([string]$env:SDKWORK_ROUTER_INSTALL_MODE) -FallbackMode $manifestInstallMode
$defaultLogDirectoryRaw = Get-RouterDefaultLogRoot -RuntimeHome $runtimeHome -InstallMode $installMode
$defaultRunDirectoryRaw = Get-RouterDefaultRunRoot -RuntimeHome $runtimeHome -InstallMode $installMode
$logDirectoryRaw = if (-not [string]::IsNullOrWhiteSpace($manifestLogDirectory)) {
    $manifestLogDirectory
} else {
    $defaultLogDirectoryRaw
}
$runDirectoryRaw = if (-not [string]::IsNullOrWhiteSpace($manifestRunDirectory)) {
    $manifestRunDirectory
} else {
    $defaultRunDirectoryRaw
}
$logDirectory = Resolve-RouterHostPath -PathValue $logDirectoryRaw -DefaultValue $defaultLogDirectoryRaw
$runDirectory = Resolve-RouterHostPath -PathValue $runDirectoryRaw -DefaultValue $defaultRunDirectoryRaw
$pidFile = Join-Path $runDirectory 'router-product-service.pid'
$stateFile = Join-Path $runDirectory 'router-product-service.state.env'
$stdoutLog = Join-Path $logDirectory 'router-product-service.stdout.log'
$stderrLog = Join-Path $logDirectory 'router-product-service.stderr.log'

if ($DryRun) {
    Write-RouterInfo "would stop router-product-service using pid file $pidFile"
    return
}

if (-not (Test-Path $pidFile)) {
    Remove-RouterManagedStateFile -StateFile $stateFile
    Write-RouterInfo "pid file not found, nothing to stop: $pidFile"
    return
}

$pidValue = Get-RouterManagedProcessId -PidFile $pidFile -StateFile $stateFile
if ($pidValue -le 0) {
    Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
    Remove-RouterManagedStateFile -StateFile $stateFile
    Write-RouterInfo "process already stopped, removed stale pid file: $pidFile"
    return
}

$stopped = Stop-RouterProcessTree -ProcessId ([int]$pidValue) -WaitSeconds $WaitSeconds -Force:(-not $GracefulOnly)
if (-not $stopped) {
    Show-RouterLogTail -LogFile $stdoutLog
    Show-RouterLogTail -LogFile $stderrLog
    Throw-RouterError "failed to stop router-product-service pid=$pidValue"
}

Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
Remove-RouterManagedStateFile -StateFile $stateFile
Write-RouterInfo "stopped router-product-service pid=$pidValue"
