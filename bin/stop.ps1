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
$runDirectory = Join-Path $runtimeHome 'var\run'
$logDirectory = Join-Path $runtimeHome 'var\log'
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
