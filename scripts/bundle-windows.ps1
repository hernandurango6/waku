param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",
    [switch]$SkipBuild,
    [switch]$SkipInstaller
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$TargetTriple = "x86_64-pc-windows-msvc"
$TargetProfile = if ($Profile -eq "release") { "release" } else { "debug" }
$TargetDir = Join-Path $Root "target\$TargetTriple\$TargetProfile"
$StageDir = Join-Path $Root "dist\windows\Waku"
$InstallerScript = Join-Path $Root "scripts\Waku.iss"

Push-Location $Root
try {
    if (-not $SkipBuild) {
        if ($Profile -eq "release") {
            cargo build --release --locked --target $TargetTriple --bin waku --bin waku_js_repl
        } else {
            cargo build --locked --target $TargetTriple --bin waku --bin waku_js_repl
        }
    }

    if (Test-Path $StageDir) {
        Remove-Item -Recurse -Force $StageDir
    }
    New-Item -ItemType Directory -Force $StageDir | Out-Null

    Copy-Item (Join-Path $TargetDir "waku.exe") $StageDir
    Copy-Item (Join-Path $TargetDir "waku_js_repl.exe") $StageDir
    Copy-Item (Join-Path $Root "LICENSE") $StageDir
    Copy-Item (Join-Path $Root "README.md") $StageDir

    $Version = (Select-String -Path (Join-Path $Root "Cargo.toml") -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
    $InstallerPath = Join-Path $Root "dist\windows\Waku-$Version-Setup.exe"
    $InstallerOutputDir = Join-Path $Root "dist\windows"
    New-Item -ItemType Directory -Force $InstallerOutputDir | Out-Null

    if (-not $SkipInstaller) {
        $Iscc = Get-Command iscc.exe -ErrorAction SilentlyContinue
        if (-not $Iscc) {
            $KnownIsccPaths = @(
                (Join-Path ${env:ProgramFiles} "Inno Setup 7\ISCC.exe"),
                (Join-Path ${env:ProgramFiles} "Inno Setup 6\ISCC.exe"),
                (Join-Path ${env:ProgramFiles(x86)} "Inno Setup 7\ISCC.exe"),
                (Join-Path ${env:ProgramFiles(x86)} "Inno Setup 6\ISCC.exe")
            )
            $KnownIscc = $KnownIsccPaths |
                Where-Object { Test-Path -LiteralPath $_ } |
                Select-Object -First 1
            if ($KnownIscc) {
                $Iscc = [pscustomobject]@{ Source = $KnownIscc }
            } else {
                throw "Inno Setup is required to build the installer. Install ISCC.exe or pass -SkipInstaller."
            }
        }
        Push-Location $Root
        try {
            & $Iscc.Source "/DAppVersion=$Version" "/DStageDir=$StageDir" "/DOutputDir=$InstallerOutputDir" $InstallerScript
        }
        finally {
            Pop-Location
        }
    }

    Write-Output $StageDir
}
finally {
    Pop-Location
}
