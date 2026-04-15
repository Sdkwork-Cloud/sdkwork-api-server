Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-RouterInfo {
    param([Parameter(Mandatory = $true)][string]$Message)
    Write-Host "[sdkwork-router] $Message"
}

function Throw-RouterError {
    param([Parameter(Mandatory = $true)][string]$Message)
    throw "[sdkwork-router] $Message"
}

function Get-RouterActiveBootstrapProfile {
    $bootstrapProfile = [string]$env:SDKWORK_BOOTSTRAP_PROFILE
    if ([string]::IsNullOrWhiteSpace($bootstrapProfile)) {
        return 'runtime configuration'
    }

    return $bootstrapProfile.Trim()
}

function Get-RouterBootstrapIdentityHintPath {
    $bootstrapDataDir = [string]$env:SDKWORK_BOOTSTRAP_DATA_DIR
    $bootstrapProfile = [string]$env:SDKWORK_BOOTSTRAP_PROFILE

    if ([string]::IsNullOrWhiteSpace($bootstrapDataDir) -or [string]::IsNullOrWhiteSpace($bootstrapProfile)) {
        return ''
    }

    $identityFile = Join-Path (Join-Path $bootstrapDataDir 'identities') "$($bootstrapProfile.Trim()).json"
    return Convert-ToRouterPortablePath -PathValue $identityFile
}

function Convert-ToRouterPortablePath {
    param([Parameter(Mandatory = $true)][string]$PathValue)
    return $PathValue.Replace('\', '/')
}

function Get-RouterScriptDirectory {
    param([Parameter(Mandatory = $true)][string]$ScriptPath)
    return Split-Path -Parent (Resolve-Path $ScriptPath)
}

function Get-RouterRepoRoot {
    param([Parameter(Mandatory = $true)][string]$ScriptDirectory)
    return Split-Path -Parent (Resolve-Path $ScriptDirectory)
}

function Get-RouterDefaultInstallHome {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)
    return Join-Path $RepoRoot 'artifacts\install\sdkwork-api-router\current'
}

function Get-RouterDefaultDevHome {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    $requestedDevHome = [string]$env:SDKWORK_ROUTER_DEV_HOME
    if (-not [string]::IsNullOrWhiteSpace($requestedDevHome)) {
        if ([System.IO.Path]::IsPathRooted($requestedDevHome)) {
            return [System.IO.Path]::GetFullPath($requestedDevHome)
        }

        return [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $requestedDevHome))
    }

    return Join-Path $RepoRoot (Join-Path 'artifacts\runtime\dev' (Get-RouterRuntimePlatformKey))
}

function Resolve-RouterCargoExecutable {
    if (Test-RouterWindowsPlatform) {
        return 'cargo.exe'
    }

    return 'cargo'
}

function Get-RouterManagedCargoTargetDir {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    $requestedTargetDir = [string]$env:CARGO_TARGET_DIR
    if (-not [string]::IsNullOrWhiteSpace($requestedTargetDir)) {
        if ([System.IO.Path]::IsPathRooted($requestedTargetDir)) {
            return [System.IO.Path]::GetFullPath($requestedTargetDir)
        }

        return [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $requestedTargetDir))
    }

    if (-not (Test-RouterWindowsPlatform)) {
        return Join-Path $RepoRoot 'target'
    }

    return Join-Path $RepoRoot 'bin\.sdkwork-target-vs2022'
}

function Resolve-RouterUsableCargoTargetDir {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    $requestedTargetDir = [string]$env:CARGO_TARGET_DIR
    $preferredTargetDir = Get-RouterManagedCargoTargetDir -RepoRoot $RepoRoot
    if (-not [string]::IsNullOrWhiteSpace($requestedTargetDir)) {
        Ensure-RouterDirectory -DirectoryPath $preferredTargetDir
        return $preferredTargetDir
    }

    $candidateDirectories = @($preferredTargetDir)
    if (Test-RouterWindowsPlatform) {
        $repoBinTargetDir = Join-Path $RepoRoot 'bin\.sdkwork-target-vs2022'
        if ($repoBinTargetDir -notin $candidateDirectories) {
            $candidateDirectories += $repoBinTargetDir
        }
    }

    $repoTargetDir = Join-Path $RepoRoot 'target'
    if ($repoTargetDir -notin $candidateDirectories) {
        $candidateDirectories += $repoTargetDir
    }

    $lastErrorMessage = ''
    foreach ($candidateDirectory in $candidateDirectories) {
        try {
            Ensure-RouterDirectory -DirectoryPath $candidateDirectory
            return $candidateDirectory
        } catch {
            $lastErrorMessage = $_.Exception.Message
        }
    }

    if ($lastErrorMessage) {
        Throw-RouterError "failed to initialize a cargo target directory: $lastErrorMessage"
    }

    Throw-RouterError 'failed to initialize a cargo target directory'
}

function Get-RouterWindowsBackendWarmupCargoArgs {
    param([string]$CargoBuildJobs = '')

    $cargoArgs = @(
        'build',
        '-p', 'admin-api-service',
        '-p', 'gateway-service',
        '-p', 'portal-api-service'
    )

    if (-not [string]::IsNullOrWhiteSpace($CargoBuildJobs)) {
        $cargoArgs += @('-j', $CargoBuildJobs)
    }

    return $cargoArgs
}

function Get-RouterWindowsBackendWarmupCommandDisplay {
    param([string]$CargoBuildJobs = '')
    $cargoArgs = Get-RouterWindowsBackendWarmupCargoArgs -CargoBuildJobs $CargoBuildJobs
    return "cargo $($cargoArgs -join ' ')"
}

function Enable-RouterManagedCargoEnv {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    $managedTargetDir = Resolve-RouterUsableCargoTargetDir -RepoRoot $RepoRoot

    $targetDirWasDefaulted = $false
    if ([string]::IsNullOrWhiteSpace([string]$env:CARGO_TARGET_DIR)) {
        $env:CARGO_TARGET_DIR = $managedTargetDir
        $targetDirWasDefaulted = $true
    }

    $buildJobsWasDefaulted = $false
    if ((Test-RouterWindowsPlatform) -and [string]::IsNullOrWhiteSpace([string]$env:CARGO_BUILD_JOBS)) {
        $env:CARGO_BUILD_JOBS = '1'
        $buildJobsWasDefaulted = $true
    }

    return [pscustomobject]@{
        Enabled = Test-RouterWindowsPlatform
        CargoTargetDir = $managedTargetDir
        CargoBuildJobs = [string]$env:CARGO_BUILD_JOBS
        CargoTargetDirWasDefaulted = $targetDirWasDefaulted
        CargoBuildJobsWasDefaulted = $buildJobsWasDefaulted
    }
}

function Invoke-RouterWindowsBackendWarmupBuild {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [string]$CargoBuildJobs = ''
    )

    if (-not (Test-RouterWindowsPlatform)) {
        return
    }

    $cargoArgs = Get-RouterWindowsBackendWarmupCargoArgs -CargoBuildJobs $CargoBuildJobs
    Write-RouterInfo "backend warm-up: $(Get-RouterWindowsBackendWarmupCommandDisplay -CargoBuildJobs $CargoBuildJobs)"

    Push-Location $RepoRoot
    try {
        & (Resolve-RouterCargoExecutable) @cargoArgs
        if ($LASTEXITCODE -ne 0) {
            Throw-RouterError "backend warm-up cargo build failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

function Test-RouterWindowsPlatform {
    $osName = [string]$env:OS
    if ($osName.Equals('Windows_NT', [System.StringComparison]::OrdinalIgnoreCase)) {
        return $true
    }

    return $PSVersionTable.PSEdition -eq 'Desktop'
}

function Get-RouterRuntimeArchitecture {
    try {
        $architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    } catch {
        $architecture = [string]$env:PROCESSOR_ARCHITECTURE
    }

    switch ($architecture.ToUpperInvariant()) {
        'AMD64' { return 'x64' }
        'X64' { return 'x64' }
        'ARM64' { return 'arm64' }
        'X86' { return 'x86' }
        'IA32' { return 'x86' }
        default { return $architecture.ToLowerInvariant() }
    }
}

function Get-RouterRuntimePlatformKey {
    $platformName = 'unknown'
    if (Test-RouterWindowsPlatform) {
        $platformName = 'windows'
    } else {
        try {
            if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)) {
                $platformName = 'linux'
            } elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
                $platformName = 'macos'
            }
        } catch {
            $platformName = 'unknown'
        }
    }

    return "$platformName-$(Get-RouterRuntimeArchitecture)"
}

function Get-RouterBinaryName {
    param([Parameter(Mandatory = $true)][string]$BaseName)

    if (Test-RouterWindowsPlatform) {
        return "$BaseName.exe"
    }

    return $BaseName
}

function Ensure-RouterDirectory {
    param([Parameter(Mandatory = $true)][string]$DirectoryPath)
    New-Item -ItemType Directory -Force -Path $DirectoryPath | Out-Null
}

function Get-RouterPnpmVirtualStoreDir {
    param([Parameter(Mandatory = $true)][string]$NodeModulesPath)

    $modulesFile = Join-Path $NodeModulesPath '.modules.yaml'
    if (-not (Test-Path $modulesFile -PathType Leaf)) {
        return ''
    }

    foreach ($rawLine in Get-Content $modulesFile -ErrorAction SilentlyContinue) {
        $line = [string]$rawLine
        if ($line -match '^\s*virtualStoreDir:\s*(?<value>.+?)\s*$') {
            return $Matches.value.Trim()
        }
    }

    return ''
}

function Resolve-RouterAbsolutePath {
    param(
        [Parameter(Mandatory = $true)][string]$BasePath,
        [Parameter(Mandatory = $true)][string]$CandidatePath
    )

    if ([System.IO.Path]::IsPathRooted($CandidatePath)) {
        return [System.IO.Path]::GetFullPath($CandidatePath)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $BasePath $CandidatePath))
}

function Test-RouterPnpmNodeModulesHealthy {
    param([Parameter(Mandatory = $true)][string]$NodeModulesPath)

    if (-not (Test-Path $NodeModulesPath -PathType Container)) {
        return $false
    }

    $expectedVirtualStoreDir = [System.IO.Path]::GetFullPath((Join-Path $NodeModulesPath '.pnpm'))
    if (-not (Test-Path $expectedVirtualStoreDir -PathType Container)) {
        return $false
    }

    $configuredVirtualStoreDir = Get-RouterPnpmVirtualStoreDir -NodeModulesPath $NodeModulesPath
    if ([string]::IsNullOrWhiteSpace($configuredVirtualStoreDir)) {
        return $false
    }

    $resolvedConfiguredVirtualStoreDir = Resolve-RouterAbsolutePath `
        -BasePath $NodeModulesPath `
        -CandidatePath $configuredVirtualStoreDir

    return $resolvedConfiguredVirtualStoreDir.Equals(
        $expectedVirtualStoreDir,
        [System.StringComparison]::OrdinalIgnoreCase
    )
}

function Test-RouterPackageManifestPresent {
    param(
        [Parameter(Mandatory = $true)][string]$NodeModulesPath,
        [Parameter(Mandatory = $true)][string]$PackageName
    )

    $manifestPath = $NodeModulesPath
    foreach ($segment in ($PackageName -split '/')) {
        $manifestPath = Join-Path $manifestPath $segment
    }
    $manifestPath = Join-Path $manifestPath 'package.json'

    return Test-Path $manifestPath -PathType Leaf
}

function Test-RouterPortalFrontendDependencyLinksUsable {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    $portalRoot = Join-Path $RepoRoot 'apps\sdkwork-router-portal'
    $portalNodeModules = Join-Path $portalRoot 'node_modules'
    if (-not (Test-Path $portalNodeModules -PathType Container)) {
        return $false
    }

    $requiredRootPackages = @(
        'react-router-dom',
        'zustand',
        'sdkwork-router-portal-commons',
        '@radix-ui/react-slot',
        '@tanstack/react-table',
        'cmdk',
        'class-variance-authority',
        'react-hook-form',
        'react-day-picker',
        'react-resizable-panels',
        'sonner'
    )

    foreach ($packageName in $requiredRootPackages) {
        if (-not (Test-RouterPackageManifestPresent -NodeModulesPath $portalNodeModules -PackageName $packageName)) {
            return $false
        }
    }

    $coreNodeModules = Join-Path $portalRoot 'packages\sdkwork-router-portal-core\node_modules'
    return Test-RouterPackageManifestPresent -NodeModulesPath $coreNodeModules -PackageName 'react-router-dom'
}

function Resolve-RouterPortalPnpmPackageRoot {
    param(
        [Parameter(Mandatory = $true)][string]$PortalNodeModulesPath,
        [Parameter(Mandatory = $true)][string]$PackagePrefix
    )

    $pnpmRoot = Join-Path $PortalNodeModulesPath '.pnpm'
    if (-not (Test-Path $pnpmRoot -PathType Container)) {
        return ''
    }

    $match = Get-ChildItem $pnpmRoot -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "$PackagePrefix@*" } |
        Sort-Object Name -Descending |
        Select-Object -First 1

    if ($null -eq $match) {
        return ''
    }

    return $match.FullName
}

function Set-RouterJunction {
    param(
        [Parameter(Mandatory = $true)][string]$LinkPath,
        [Parameter(Mandatory = $true)][string]$TargetPath
    )

    if (-not (Test-Path $TargetPath -PathType Container)) {
        return $false
    }

    $parentPath = Split-Path -Parent $LinkPath
    Ensure-RouterDirectory -DirectoryPath $parentPath

    try {
        $existing = Get-Item -LiteralPath $LinkPath -Force -ErrorAction SilentlyContinue
        if ($existing) {
            $hasPackageManifest = Test-Path (Join-Path $LinkPath 'package.json') -PathType Leaf
            if ($hasPackageManifest) {
                return $true
            }

            Remove-Item -LiteralPath $LinkPath -Force -ErrorAction SilentlyContinue
        }
        New-Item -ItemType Junction -Path $LinkPath -Target ([System.IO.Path]::GetFullPath($TargetPath)) | Out-Null
        return $true
    } catch {
        return $false
    }
}

function Repair-RouterPortalFrontendNodeModules {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    if (-not (Test-RouterWindowsPlatform)) {
        return $false
    }

    $portalRoot = Join-Path $RepoRoot 'apps\sdkwork-router-portal'
    $portalNodeModules = Join-Path $portalRoot 'node_modules'
    $uiNodeModules = Join-Path (Split-Path -Parent $RepoRoot) 'sdkwork-ui\sdkwork-ui-pc-react\node_modules'

    if (-not (Test-Path $portalNodeModules -PathType Container) -or -not (Test-Path $uiNodeModules -PathType Container)) {
        return $false
    }

    $reactRouterRoot = Resolve-RouterPortalPnpmPackageRoot `
        -PortalNodeModulesPath $portalNodeModules `
        -PackagePrefix 'react-router-dom'
    $zustandRoot = Resolve-RouterPortalPnpmPackageRoot `
        -PortalNodeModulesPath $portalNodeModules `
        -PackagePrefix 'zustand'

    if ([string]::IsNullOrWhiteSpace($reactRouterRoot) -or [string]::IsNullOrWhiteSpace($zustandRoot)) {
        return $false
    }

    $rootLinks = [ordered]@{
        'react-router-dom' = Join-Path $reactRouterRoot 'node_modules\react-router-dom'
        'zustand' = Join-Path $zustandRoot 'node_modules\zustand'
        'sdkwork-router-portal-commons' = Join-Path $portalRoot 'packages\sdkwork-router-portal-commons'
        '@radix-ui' = Join-Path $uiNodeModules '@radix-ui'
        '@tanstack' = Join-Path $uiNodeModules '@tanstack'
        'cmdk' = Join-Path $uiNodeModules 'cmdk'
        'class-variance-authority' = Join-Path $uiNodeModules 'class-variance-authority'
        'react-hook-form' = Join-Path $uiNodeModules 'react-hook-form'
        'react-day-picker' = Join-Path $uiNodeModules 'react-day-picker'
        'react-resizable-panels' = Join-Path $uiNodeModules 'react-resizable-panels'
        'sonner' = Join-Path $uiNodeModules 'sonner'
    }

    $repaired = $false
    foreach ($linkName in $rootLinks.Keys) {
        if (Set-RouterJunction -LinkPath (Join-Path $portalNodeModules $linkName) -TargetPath $rootLinks[$linkName]) {
            $repaired = $true
        }
    }

    $rootReactRouterPath = Join-Path $portalNodeModules 'react-router-dom'
    foreach ($packageDirectory in Get-ChildItem (Join-Path $portalRoot 'packages') -Directory -ErrorAction SilentlyContinue) {
        $packageNodeModules = Join-Path $packageDirectory.FullName 'node_modules'
        if (-not (Test-Path $packageNodeModules -PathType Container)) {
            continue
        }

        if (Set-RouterJunction -LinkPath (Join-Path $packageNodeModules 'react-router-dom') -TargetPath $rootReactRouterPath) {
            $repaired = $true
        }
    }

    return $repaired
}

function ConvertTo-RouterFileText {
    param([AllowNull()][object]$Content)

    if ($null -eq $Content) {
        return ''
    }

    if ($Content -is [System.Array]) {
        $lines = foreach ($entry in $Content) {
            if ($null -eq $entry) {
                ''
            } else {
                [string]$entry
            }
        }

        return [string]::Join([Environment]::NewLine, $lines)
    }

    return [string]$Content
}

function Write-RouterUtf8File {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [AllowNull()][object]$Content,
        [int]$RetryCount = 4,
        [int]$RetryDelayMs = 75
    )

    $directory = Split-Path -Parent $FilePath
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        Ensure-RouterDirectory -DirectoryPath $directory
    }

    $payload = ConvertTo-RouterFileText -Content $Content
    $encoding = [System.Text.UTF8Encoding]::new($false)
    $lastException = $null

    for ($attempt = 1; $attempt -le $RetryCount; $attempt++) {
        try {
            [System.IO.File]::WriteAllText($FilePath, $payload, $encoding)
            return
        } catch [System.UnauthorizedAccessException] {
            $lastException = $_.Exception
        } catch [System.IO.IOException] {
            $lastException = $_.Exception
        }

        if ($attempt -lt $RetryCount) {
            Start-Sleep -Milliseconds $RetryDelayMs
        }
    }

    if ($null -ne $lastException) {
        throw $lastException
    }

    Throw-RouterError "failed to write file: $FilePath"
}

function Test-RouterWindowsStylePath {
    param([Parameter(Mandatory = $true)][string]$PathValue)
    return $PathValue -match '^[A-Za-z]:[\\/]' -or $PathValue -match '^(\\\\|//)'
}

function Test-RouterUnixAbsolutePath {
    param([Parameter(Mandatory = $true)][string]$PathValue)
    return $PathValue.StartsWith('/')
}

function Test-RouterPathHostCompatible {
    param([Parameter(Mandatory = $true)][string]$PathValue)
    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        return $true
    }

    if (Test-RouterWindowsPlatform) {
        if ((Test-RouterUnixAbsolutePath -PathValue $PathValue) -and -not (Test-RouterWindowsStylePath -PathValue $PathValue)) {
            return $false
        }
        return $true
    }

    return -not (Test-RouterWindowsStylePath -PathValue $PathValue)
}

function Resolve-RouterHostPath {
    param(
        [Parameter(Mandatory = $true)][string]$PathValue,
        [Parameter(Mandatory = $true)][string]$DefaultValue
    )

    if (Test-RouterPathHostCompatible -PathValue $PathValue) {
        return $PathValue
    }

    return $DefaultValue
}

function Test-RouterDatabaseUrlHostCompatible {
    param([Parameter(Mandatory = $true)][string]$DatabaseUrl)
    if ([string]::IsNullOrWhiteSpace($DatabaseUrl) -or -not $DatabaseUrl.StartsWith('sqlite://', [System.StringComparison]::OrdinalIgnoreCase)) {
        return $true
    }

    $databasePath = $DatabaseUrl.Substring('sqlite://'.Length)
    return Test-RouterPathHostCompatible -PathValue $databasePath
}

function Resolve-RouterHostDatabaseUrl {
    param(
        [Parameter(Mandatory = $true)][string]$DatabaseUrl,
        [Parameter(Mandatory = $true)][string]$DefaultValue
    )

    if (Test-RouterDatabaseUrlHostCompatible -DatabaseUrl $DatabaseUrl) {
        return $DatabaseUrl
    }

    return $DefaultValue
}

function ConvertFrom-RouterBindAddress {
    param([Parameter(Mandatory = $true)][string]$BindAddress)

    if ($BindAddress -notmatch '^(?<host>\[[^\]]+\]|.+):(?<port>\d+)$') {
        Throw-RouterError "invalid bind address: $BindAddress"
    }

    $bindHost = $Matches.host
    if ($bindHost.StartsWith('[') -and $bindHost.EndsWith(']')) {
        $bindHost = $bindHost.Substring(1, $bindHost.Length - 2)
    }

    $bindPort = 0
    if (-not [int]::TryParse($Matches.port, [ref]$bindPort) -or $bindPort -lt 1 -or $bindPort -gt 65535) {
        Throw-RouterError "invalid bind address: $BindAddress"
    }

    return [pscustomobject]@{
        BindAddress = $BindAddress
        Host = $bindHost
        Port = $bindPort
    }
}

function Resolve-RouterBindIpAddresses {
    param([Parameter(Mandatory = $true)][string]$BindHost)

    if ([string]::IsNullOrWhiteSpace($BindHost) -or $BindHost -eq '0.0.0.0') {
        return @([System.Net.IPAddress]::Any)
    }

    if ($BindHost -eq '::' -or $BindHost -eq '[::]') {
        return @([System.Net.IPAddress]::IPv6Any)
    }

    $ipAddress = $null
    if ([System.Net.IPAddress]::TryParse($BindHost, [ref]$ipAddress)) {
        return @($ipAddress)
    }

    try {
        $resolvedAddresses = [System.Net.Dns]::GetHostAddresses($BindHost)
    } catch {
        Throw-RouterError "failed to resolve bind host '$BindHost': $($_.Exception.Message)"
    }

    if (-not $resolvedAddresses -or $resolvedAddresses.Count -eq 0) {
        Throw-RouterError "failed to resolve bind host '$BindHost'"
    }

    return @($resolvedAddresses | Select-Object -Unique)
}

function Test-RouterBindAddressAvailability {
    param([Parameter(Mandatory = $true)][string]$BindAddress)

    $parsedBind = ConvertFrom-RouterBindAddress -BindAddress $BindAddress

    foreach ($ipAddress in Resolve-RouterBindIpAddresses -BindHost $parsedBind.Host) {
        $socket = $null
        try {
            $socket = [System.Net.Sockets.Socket]::new(
                $ipAddress.AddressFamily,
                [System.Net.Sockets.SocketType]::Stream,
                [System.Net.Sockets.ProtocolType]::Tcp
            )
            $socket.ExclusiveAddressUse = $true
            $socket.Bind([System.Net.IPEndPoint]::new($ipAddress, $parsedBind.Port))
        } catch [System.Net.Sockets.SocketException] {
            $reason = if ($_.Exception.SocketErrorCode -eq [System.Net.Sockets.SocketError]::AddressAlreadyInUse) {
                'address already in use'
            } else {
                $_.Exception.Message
            }

            return [pscustomobject]@{
                BindAddress = $BindAddress
                Available = $false
                Reason = $reason
            }
        } finally {
            if ($null -ne $socket) {
                $socket.Dispose()
            }
        }
    }

    return [pscustomobject]@{
        BindAddress = $BindAddress
        Available = $true
        Reason = ''
    }
}

function Get-RouterListeningPortConflicts {
    param([Parameter(Mandatory = $true)][string[]]$BindAddresses)

    $conflicts = @()
    foreach ($bindAddress in $BindAddresses) {
        if ([string]::IsNullOrWhiteSpace($bindAddress)) {
            continue
        }

        $availability = Test-RouterBindAddressAvailability -BindAddress $bindAddress
        if (-not $availability.Available) {
            $conflicts += $availability
        }
    }

    return @($conflicts)
}

function Assert-RouterBindAddressesAvailable {
    param(
        [Parameter(Mandatory = $true)][string[]]$BindAddresses,
        [string]$ServiceLabel = 'service'
    )

    $conflicts = @(Get-RouterListeningPortConflicts -BindAddresses $BindAddresses)
    if ($conflicts.Count -eq 0) {
        return
    }

    $messageLines = @("$ServiceLabel cannot start because required listen ports are already in use:")
    foreach ($conflict in $conflicts) {
        $messageLines += "  $($conflict.BindAddress) ($($conflict.Reason))"
    }
    $messageLines += 'Stop the conflicting process or override the bind addresses before retrying.'

    Throw-RouterError ($messageLines -join [Environment]::NewLine)
}

function ConvertTo-RouterRoleList {
    param([string]$RolesValue = '')

    $roles = @()
    foreach ($rawRole in ($RolesValue -split '[,;]')) {
        $role = $rawRole.Trim()
        if ($role) {
            $roles += $role
        }
    }

    if ($roles.Count -eq 0) {
        return @('web', 'gateway', 'admin', 'portal')
    }

    return $roles
}

function Get-RouterReleaseDryRunPlanJson {
    param(
        [Parameter(Mandatory = $true)][string]$ConfigDir,
        [Parameter(Mandatory = $true)][string]$DatabaseUrl,
        [Parameter(Mandatory = $true)][string]$WebBind,
        [Parameter(Mandatory = $true)][string]$GatewayBind,
        [Parameter(Mandatory = $true)][string]$AdminBind,
        [Parameter(Mandatory = $true)][string]$PortalBind,
        [string]$ConfigFile = '',
        [string]$Roles = '',
        [string]$NodeIdPrefix = '',
        [string]$GatewayUpstream = '',
        [string]$AdminUpstream = '',
        [string]$PortalUpstream = '',
        [Parameter(Mandatory = $true)][string]$AdminSiteDir,
        [Parameter(Mandatory = $true)][string]$PortalSiteDir
    )

    $plan = [ordered]@{
        mode = 'dry-run'
        plan_format = 'json'
        roles = @(ConvertTo-RouterRoleList -RolesValue $Roles)
        public_web_bind = $WebBind
        database_url = $DatabaseUrl
        config_dir = $ConfigDir
        config_file = if ($ConfigFile) { $ConfigFile } else { $null }
        node_id_prefix = if ($NodeIdPrefix) { $NodeIdPrefix } else { $null }
        binds = [ordered]@{
            gateway = $GatewayBind
            admin = $AdminBind
            portal = $PortalBind
        }
        site_dirs = [ordered]@{
            admin = $AdminSiteDir
            portal = $PortalSiteDir
        }
        upstreams = [ordered]@{
            gateway = if ($GatewayUpstream) { $GatewayUpstream } else { $null }
            admin = if ($AdminUpstream) { $AdminUpstream } else { $null }
            portal = if ($PortalUpstream) { $PortalUpstream } else { $null }
        }
    }

    return $plan | ConvertTo-Json -Depth 4
}

function Import-RouterEnvFile {
    param([Parameter(Mandatory = $true)][string]$EnvFile)
    if (-not (Test-Path $EnvFile)) {
        return
    }

    foreach ($rawLine in Get-Content $EnvFile) {
        $line = $rawLine.Trim()
        if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith('#')) {
            continue
        }

        $separatorIndex = $line.IndexOf('=')
        if ($separatorIndex -lt 1) {
            continue
        }

        $key = $line.Substring(0, $separatorIndex).Trim()
        $value = $line.Substring($separatorIndex + 1).Trim()
        if ($value.Length -ge 2) {
            $quote = $value[0]
            if (($quote -eq '"' -or $quote -eq "'") -and $value[-1] -eq $quote) {
                $value = $value.Substring(1, $value.Length - 2)
                if ($quote -eq '"') {
                    $value = $value.Replace('\"', '"').Replace('\\', '\')
                }
            }
        }
        Set-Item -Path "Env:$key" -Value $value
    }
}

function Test-RouterProcessRunning {
    param([Parameter(Mandatory = $true)][string]$PidValue)
    if ([string]::IsNullOrWhiteSpace($PidValue)) {
        return $false
    }

    $process = Get-Process -Id ([int]$PidValue) -ErrorAction SilentlyContinue
    return $null -ne $process
}

function Get-RouterProcessFingerprint {
    param([Parameter(Mandatory = $true)][int]$ProcessId)

    $process = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
    if ($null -eq $process) {
        return ''
    }

    try {
        return $process.StartTime.ToUniversalTime().ToString('o')
    } catch {
        return ''
    }
}

function Get-RouterPidFileValue {
    param([Parameter(Mandatory = $true)][string]$PidFile)

    if (-not (Test-Path $PidFile)) {
        return ''
    }

    $rawPidValue = Get-Content $PidFile -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $rawPidValue) {
        return ''
    }

    return ([string]$rawPidValue).Trim()
}

function Remove-RouterManagedStateFile {
    param([string]$StateFile = '')

    if ([string]::IsNullOrWhiteSpace($StateFile)) {
        return
    }

    Remove-Item $StateFile -Force -ErrorAction SilentlyContinue
}

function ConvertTo-RouterStateFileLine {
    param(
        [Parameter(Mandatory = $true)][string]$Key,
        [Parameter(Mandatory = $true)][AllowEmptyString()][string]$Value
    )

    $escapedValue = $Value.Replace('\', '\\').Replace('"', '\"')
    return "$Key=`"$escapedValue`""
}

function Get-RouterManagedStateValue {
    param(
        [Parameter(Mandatory = $true)][hashtable]$State,
        [Parameter(Mandatory = $true)][string]$Key
    )

    if (-not $State.ContainsKey($Key)) {
        return ''
    }

    return [string]$State[$Key]
}

function Get-RouterManagedState {
    param([string]$StateFile = '')

    if ([string]::IsNullOrWhiteSpace($StateFile) -or -not (Test-Path $StateFile)) {
        return $null
    }

    $state = @{}
    foreach ($rawLine in Get-Content $StateFile -ErrorAction SilentlyContinue) {
        $line = ([string]$rawLine).Trim()
        if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith('#')) {
            continue
        }

        $separatorIndex = $line.IndexOf('=')
        if ($separatorIndex -lt 1) {
            continue
        }

        $key = $line.Substring(0, $separatorIndex).Trim()
        $value = $line.Substring($separatorIndex + 1).Trim()
        if ($value.Length -ge 2) {
            $quote = $value[0]
            if (($quote -eq '"' -or $quote -eq "'") -and $value[-1] -eq $quote) {
                $value = $value.Substring(1, $value.Length - 2)
                if ($quote -eq '"') {
                    $value = $value.Replace('\"', '"').Replace('\\', '\')
                }
            }
        }

        $state[$key] = $value
    }

    if ($state.Count -eq 0) {
        return $null
    }

    $processId = 0
    [void][int]::TryParse((Get-RouterManagedStateValue -State $state -Key 'SDKWORK_ROUTER_MANAGED_PID'), [ref]$processId)

    $unifiedAccessEnabled = $false
    if ($state.ContainsKey('SDKWORK_ROUTER_UNIFIED_ACCESS_ENABLED')) {
        [void][bool]::TryParse(
            (Get-RouterManagedStateValue -State $state -Key 'SDKWORK_ROUTER_UNIFIED_ACCESS_ENABLED'),
            [ref]$unifiedAccessEnabled
        )
    }

    return [pscustomobject]@{
        ProcessId = $processId
        ProcessFingerprint = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_ROUTER_PROCESS_FINGERPRINT'
        Mode = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_ROUTER_MODE'
        WebBind = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_WEB_BIND'
        GatewayBind = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_GATEWAY_BIND'
        AdminBind = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_ADMIN_BIND'
        PortalBind = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_PORTAL_BIND'
        UnifiedAccessEnabled = $unifiedAccessEnabled
        AdminAppUrl = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_ROUTER_ADMIN_APP_URL'
        PortalAppUrl = Get-RouterManagedStateValue -State $state -Key 'SDKWORK_ROUTER_PORTAL_APP_URL'
    }
}

function Write-RouterManagedStateFile {
    param(
        [Parameter(Mandatory = $true)][string]$StateFile,
        [Parameter(Mandatory = $true)][int]$ProcessId,
        [string]$ProcessFingerprint = '',
        [Parameter(Mandatory = $true)][string]$Mode,
        [Parameter(Mandatory = $true)][string]$WebBind,
        [Parameter(Mandatory = $true)][string]$GatewayBind,
        [Parameter(Mandatory = $true)][string]$AdminBind,
        [Parameter(Mandatory = $true)][string]$PortalBind,
        [bool]$UnifiedAccessEnabled = $true,
        [string]$AdminAppUrl = '',
        [string]$PortalAppUrl = ''
    )

    $directory = Split-Path -Parent $StateFile
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        Ensure-RouterDirectory -DirectoryPath $directory
    }

    $lines = @(
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_ROUTER_MANAGED_PID' -Value ([string]$ProcessId)),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_ROUTER_PROCESS_FINGERPRINT' -Value $ProcessFingerprint),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_ROUTER_MODE' -Value $Mode),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_WEB_BIND' -Value $WebBind),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_GATEWAY_BIND' -Value $GatewayBind),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_ADMIN_BIND' -Value $AdminBind),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_PORTAL_BIND' -Value $PortalBind),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_ROUTER_UNIFIED_ACCESS_ENABLED' -Value ([string]$UnifiedAccessEnabled)),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_ROUTER_ADMIN_APP_URL' -Value $AdminAppUrl),
        (ConvertTo-RouterStateFileLine -Key 'SDKWORK_ROUTER_PORTAL_APP_URL' -Value $PortalAppUrl)
    )

    Write-RouterUtf8File -FilePath $StateFile -Content $lines
}

function Clear-RouterStalePidFile {
    param(
        [Parameter(Mandatory = $true)][string]$PidFile,
        [string]$StateFile = ''
    )
    if (-not (Test-Path $PidFile)) {
        Remove-RouterManagedStateFile -StateFile $StateFile
        return $true
    }

    $pidValue = Get-RouterPidFileValue -PidFile $PidFile
    if ([string]::IsNullOrWhiteSpace($pidValue)) {
        Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $StateFile
        return $true
    }

    if (-not (Test-RouterProcessRunning -PidValue $pidValue)) {
        Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $StateFile
        return $true
    }

    if (-not [string]::IsNullOrWhiteSpace($StateFile)) {
        $managedState = Get-RouterManagedState -StateFile $StateFile
        if ($null -eq $managedState -or $managedState.ProcessId -ne [int]$pidValue) {
            Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
            Remove-RouterManagedStateFile -StateFile $StateFile
            return $true
        }

        $currentFingerprint = Get-RouterProcessFingerprint -ProcessId ([int]$pidValue)
        if (-not [string]::IsNullOrWhiteSpace($managedState.ProcessFingerprint) -and $managedState.ProcessFingerprint -ne $currentFingerprint) {
            Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
            Remove-RouterManagedStateFile -StateFile $StateFile
            return $true
        }
    }

    return $false
}

function Get-RouterManagedProcessId {
    param(
        [Parameter(Mandatory = $true)][string]$PidFile,
        [string]$StateFile = ''
    )

    if (-not (Test-Path $PidFile)) {
        Remove-RouterManagedStateFile -StateFile $StateFile
        return 0
    }

    $pidValue = Get-RouterPidFileValue -PidFile $PidFile
    if ([string]::IsNullOrWhiteSpace($pidValue)) {
        Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $StateFile
        return 0
    }

    if (-not (Test-RouterProcessRunning -PidValue $pidValue)) {
        Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
        Remove-RouterManagedStateFile -StateFile $StateFile
        return 0
    }

    if (-not [string]::IsNullOrWhiteSpace($StateFile)) {
        $managedState = Get-RouterManagedState -StateFile $StateFile
        if ($null -eq $managedState -or $managedState.ProcessId -ne [int]$pidValue) {
            Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
            Remove-RouterManagedStateFile -StateFile $StateFile
            return 0
        }

        $currentFingerprint = Get-RouterProcessFingerprint -ProcessId ([int]$pidValue)
        if (-not [string]::IsNullOrWhiteSpace($managedState.ProcessFingerprint) -and $managedState.ProcessFingerprint -ne $currentFingerprint) {
            Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
            Remove-RouterManagedStateFile -StateFile $StateFile
            return 0
        }
    }

    return [int]$pidValue
}

function Assert-RouterNotRunning {
    param(
        [Parameter(Mandatory = $true)][string]$PidFile,
        [string]$StateFile = ''
    )
    $pidValue = Get-RouterManagedProcessId -PidFile $PidFile -StateFile $StateFile
    if ($pidValue -le 0) {
        return
    }

    Throw-RouterError "process already running with pid $pidValue (pid file: $PidFile)"
}

function Wait-RouterProcessExit {
    param(
        [Parameter(Mandatory = $true)][int]$ProcessId,
        [Parameter(Mandatory = $true)][int]$WaitSeconds
    )

    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
            return $true
        }
        Start-Sleep -Seconds 1
    }

    return -not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
}

function Confirm-RouterProcessAlive {
    param(
        [Parameter(Mandatory = $true)][int]$ProcessId,
        [int]$WaitSeconds = 2
    )

    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
            return $false
        }

        Start-Sleep -Milliseconds 250
    }

    return $true
}

function Get-RouterChildProcessIds {
    param([Parameter(Mandatory = $true)][int]$ParentPid)

    if (Test-RouterWindowsPlatform) {
        $cimCommand = Get-Command Get-CimInstance -ErrorAction SilentlyContinue
        if ($null -eq $cimCommand) {
            return @()
        }

        $childProcesses = Get-CimInstance Win32_Process -Filter "ParentProcessId = $ParentPid" -ErrorAction SilentlyContinue
        return @($childProcesses | ForEach-Object { [int]$_.ProcessId })
    }

    $psCommand = Get-Command ps -ErrorAction SilentlyContinue
    if ($null -eq $psCommand) {
        return @()
    }

    $childIds = @()
    foreach ($line in (& ps -o pid= -o ppid= 2>$null)) {
        $parts = @($line -split '\s+' | Where-Object { $_ })
        if ($parts.Count -lt 2) {
            continue
        }

        $processId = 0
        $reportedParentPid = 0
        if (-not [int]::TryParse($parts[0], [ref]$processId)) {
            continue
        }
        if (-not [int]::TryParse($parts[1], [ref]$reportedParentPid)) {
            continue
        }
        if ($reportedParentPid -eq $ParentPid) {
            $childIds += $processId
        }
    }

    return @($childIds | Select-Object -Unique)
}

function Get-RouterProcessTreeIds {
    param([Parameter(Mandatory = $true)][int]$ParentPid)

    $descendants = @()
    foreach ($childPid in Get-RouterChildProcessIds -ParentPid $ParentPid) {
        $descendants += $childPid
        $descendants += Get-RouterProcessTreeIds -ParentPid $childPid
    }

    return @($descendants | Select-Object -Unique)
}

function Stop-RouterProcessTree {
    param(
        [Parameter(Mandatory = $true)][int]$ProcessId,
        [Parameter(Mandatory = $true)][int]$WaitSeconds,
        [switch]$Force
    )

    if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
        return $true
    }

    if (Test-RouterWindowsPlatform) {
        try {
            & cmd.exe /c "taskkill /PID $ProcessId /T >nul 2>nul" | Out-Null
        } catch {
        }
        if (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds) {
            return $true
        }

        if (-not $Force) {
            return $false
        }

        try {
            & cmd.exe /c "taskkill /PID $ProcessId /T /F >nul 2>nul" | Out-Null
        } catch {
        }
        return (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds)
    }

    $processIds = @(Get-RouterProcessTreeIds -ParentPid $ProcessId)
    $processIds += $ProcessId
    $orderedProcessIds = @($processIds | Select-Object -Unique | Sort-Object -Descending)

    foreach ($processId in $orderedProcessIds) {
        Stop-Process -Id $processId -ErrorAction SilentlyContinue
    }
    if (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds) {
        return $true
    }

    if (-not $Force) {
        return $false
    }

    foreach ($processId in $orderedProcessIds) {
        Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
    }

    return (Wait-RouterProcessExit -ProcessId $ProcessId -WaitSeconds $WaitSeconds)
}

function Start-RouterBackgroundProcess {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter()][string[]]$ArgumentList = @(),
        [Parameter(Mandatory = $true)][string]$WorkingDirectory,
        [Parameter(Mandatory = $true)][string]$StdoutLog,
        [Parameter(Mandatory = $true)][string]$StderrLog
    )

    $startProcessArgs = @{
        FilePath = $FilePath
        WorkingDirectory = $WorkingDirectory
        RedirectStandardOutput = $StdoutLog
        RedirectStandardError = $StderrLog
        NoNewWindow = $true
        PassThru = $true
    }

    if ($ArgumentList.Count -gt 0) {
        $startProcessArgs.ArgumentList = $ArgumentList
    }

    return Start-Process @startProcessArgs
}

function Resolve-RouterHealthUrl {
    param(
        [Parameter(Mandatory = $true)][string]$BindAddress,
        [Parameter(Mandatory = $true)][string]$PathSuffix
    )

    $parts = $BindAddress.Split(':')
    if ($parts.Length -lt 2) {
        Throw-RouterError "invalid bind address: $BindAddress"
    }

    $bindHost = ($parts[0..($parts.Length - 2)] -join ':')
    $bindPort = $parts[-1]
    if ([string]::IsNullOrWhiteSpace($bindHost) -or $bindHost -eq '0.0.0.0' -or $bindHost -eq '[::]' -or $bindHost -eq '::') {
        $bindHost = '127.0.0.1'
    }

    return "http://$bindHost`:$bindPort$PathSuffix"
}

function Wait-RouterHealthUrl {
    param(
        [Parameter(Mandatory = $true)][string]$Url,
        [Parameter(Mandatory = $true)][int]$WaitSeconds,
        [int]$ProcessId = 0
    )

    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        if ($ProcessId -gt 0 -and -not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
            return $false
        }
        try {
            $response = Invoke-WebRequest -UseBasicParsing $Url -TimeoutSec 3
            if ($response.StatusCode -ge 200 -and $response.StatusCode -lt 300) {
                return $true
            }
        } catch {
        }
        Start-Sleep -Seconds 1
    }

    if ($ProcessId -gt 0 -and -not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
        return $false
    }

    return $false
}

function Show-RouterLogTail {
    param([Parameter(Mandatory = $true)][string]$LogFile)
    if (Test-Path $LogFile) {
        Get-Content $LogFile -Tail 60 -ErrorAction SilentlyContinue
    }
}

function Assert-RouterFileExists {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$FilePath
    )
    if (-not (Test-Path $FilePath -PathType Leaf)) {
        Throw-RouterError "$Label not found: $FilePath"
    }
}

function Assert-RouterDirectoryExists {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$DirectoryPath
    )
    if (-not (Test-Path $DirectoryPath -PathType Container)) {
        Throw-RouterError "$Label not found: $DirectoryPath"
    }
}

function Get-RouterStartupSummaryLines {
    param(
        [Parameter(Mandatory = $true)][string]$Mode,
        [Parameter(Mandatory = $true)][string]$WebBind,
        [Parameter(Mandatory = $true)][string]$GatewayBind,
        [Parameter(Mandatory = $true)][string]$AdminBind,
        [Parameter(Mandatory = $true)][string]$PortalBind,
        [bool]$UnifiedAccessEnabled = $true,
        [string]$AdminAppUrl = '',
        [string]$PortalAppUrl = '',
        [Parameter(Mandatory = $true)][string]$StdoutLog,
        [Parameter(Mandatory = $true)][string]$StderrLog
    )

    if (-not $AdminAppUrl) {
        $AdminAppUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/admin/'
    }
    if (-not $PortalAppUrl) {
        $PortalAppUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/portal/'
    }

    $gatewayUnifiedUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/api/v1/health'
    $adminUnifiedUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/api/admin/health'
    $portalUnifiedUrl = Resolve-RouterHealthUrl -BindAddress $WebBind -PathSuffix '/api/portal/health'
    $gatewayDirectUrl = Resolve-RouterHealthUrl -BindAddress $GatewayBind -PathSuffix '/health'
    $adminDirectUrl = Resolve-RouterHealthUrl -BindAddress $AdminBind -PathSuffix '/admin/health'
    $portalDirectUrl = Resolve-RouterHealthUrl -BindAddress $PortalBind -PathSuffix '/portal/health'
    $bootstrapProfile = Get-RouterActiveBootstrapProfile
    $bootstrapIdentityHintPath = Get-RouterBootstrapIdentityHintPath

    $lines = @(
        '------------------------------------------------------------',
        "Mode: $Mode",
        "Bind Summary: web=$WebBind gateway=$GatewayBind admin=$AdminBind portal=$PortalBind"
    )

    if ($UnifiedAccessEnabled) {
        $lines += @(
            'Unified Access',
            "  Admin App: $AdminAppUrl",
            "  Portal App: $PortalAppUrl",
            "  Gateway API Health: $gatewayUnifiedUrl",
            "  Admin API Health: $adminUnifiedUrl",
            "  Portal API Health: $portalUnifiedUrl"
        )
    } else {
        $lines += @(
            'Frontend Access',
            "  Admin App: $AdminAppUrl",
            "  Portal App: $PortalAppUrl"
        )
    }

    $lines += @(
        'Direct Service Access',
        "  Gateway Service: $gatewayDirectUrl",
        "  Admin Service: $adminDirectUrl",
        "  Portal Service: $portalDirectUrl",
        'Identity Bootstrap',
        "  Local access uses the active bootstrap profile: $bootstrapProfile"
    )

    if ($bootstrapIdentityHintPath) {
        $lines += "  Review your runtime configuration and provisioned identities in $bootstrapIdentityHintPath before sharing the environment."
    } else {
        $lines += '  Review your runtime configuration and provisioned identity store before sharing the environment.'
    }

    $lines += @(
        '  Portal sign-in: use a provisioned portal user or register through /portal/auth/register.',
        '  Gateway API: sign in through the portal and create an API key.',
        'Logs',
        "  STDOUT: $StdoutLog",
        "  STDERR: $StderrLog"
    )

    return $lines
}

function Write-RouterStartupSummary {
    param(
        [Parameter(Mandatory = $true)][string]$Mode,
        [Parameter(Mandatory = $true)][string]$WebBind,
        [Parameter(Mandatory = $true)][string]$GatewayBind,
        [Parameter(Mandatory = $true)][string]$AdminBind,
        [Parameter(Mandatory = $true)][string]$PortalBind,
        [bool]$UnifiedAccessEnabled = $true,
        [string]$AdminAppUrl = '',
        [string]$PortalAppUrl = '',
        [Parameter(Mandatory = $true)][string]$StdoutLog,
        [Parameter(Mandatory = $true)][string]$StderrLog
    )

    foreach ($line in Get-RouterStartupSummaryLines @PSBoundParameters) {
        Write-RouterInfo $line
    }
}
