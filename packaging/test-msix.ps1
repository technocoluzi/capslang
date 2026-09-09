<#
.SYNOPSIS
    Registers the staged MSIX layout and exercises the code paths that only
    exist inside a package.

.DESCRIPTION
    Everything CapsLang does differently when packaged is invisible to a normal
    build: is_packaged() flips, the tray menu swaps its startup checkbox for a
    link into Settings, --autostart refuses, and start-at-login comes from the
    manifest rather than the Run key. None of that is reachable without
    actually registering a package, so this does.

    Registering an unsigned loose layout requires Developer Mode, which is why
    this runs on a CI runner -- where we are administrator and the machine is
    discarded afterwards -- rather than on a development box.
#>
[CmdletBinding()]
param(
    [string]$Layout = "dist\msix-layout",
    [string]$UnpackagedExe = "target\release\capslang.exe"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Push-Location $root

$failures = @()
function Check($name, $expected, $actual) {
    if ($actual -eq $expected) {
        Write-Host "  PASS  $name" -ForegroundColor Green
    } else {
        Write-Host "  FAIL  $name" -ForegroundColor Red
        Write-Host "        expected: $expected"
        Write-Host "        actual:   $actual"
        $script:failures += $name
    }
}

try {
    Write-Host "== Developer Mode =="
    $key = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock"
    New-Item -Path $key -Force | Out-Null
    Set-ItemProperty $key -Name AllowDevelopmentWithoutDevLicense -Value 1 -Type DWord
    Write-Host "  enabled"

    Write-Host "== Register =="
    $manifestPath = Join-Path $root "$Layout\AppxManifest.xml"
    if (-not (Test-Path $manifestPath)) { throw "no staged layout at $manifestPath" }
    Add-AppxPackage -Register $manifestPath
    $pkg = Get-AppxPackage | Where-Object { $_.Name -like "*CapsLang*" } | Select-Object -First 1
    if (-not $pkg) { throw "the package did not register" }
    Write-Host "  $($pkg.PackageFullName)"
    Write-Host "  family: $($pkg.PackageFamilyName)"

    # Registration is a much stricter reader of the manifest than makeappx:
    # it resolves the assets, the capabilities and the extensions for real.
    Write-Host "== The startup task reached Windows =="
    $ext = (Get-AppxPackageManifest $pkg.PackageFullName).Package.Applications.Application.Extensions.Extension |
        Where-Object { $_.Category -eq "windows.startupTask" }
    Check "startupTask extension present" $true ($null -ne $ext)
    if ($ext) {
        Check "startupTask TaskId" "CapsLangStartup" $ext.StartupTask.TaskId
        Check "startupTask enabled by default" "true" "$($ext.StartupTask.Enabled)".ToLower()
    }

    Write-Host "== The app knows it is packaged =="
    # A process only carries package identity when it is launched inside the
    # package, so the exe cannot simply be run from its install folder.
    $out = Join-Path $env:RUNNER_TEMP "capslang-packaged-status.txt"
    if (Test-Path $out) { Remove-Item $out -Force }
    $exe = Join-Path $pkg.InstallLocation "capslang.exe"
    Invoke-CommandInDesktopPackage -PackageFamilyName $pkg.PackageFamilyName `
        -AppId "CapsLang" `
        -Command "$env:ComSpec" `
        -Args "/c `"`"$exe`" --status > `"$out`" 2>&1`""
    for ($i = 0; $i -lt 20 -and -not (Test-Path $out); $i++) { Start-Sleep -Milliseconds 500 }
    if (-not (Test-Path $out)) { throw "the packaged app produced no output" }
    $status = (Get-Content $out -Raw).Trim()
    Write-Host "--- packaged --status ---"
    Write-Host $status
    Write-Host "-------------------------"
    Check "reports MSIX package" $true ($status -match "Installed as:\s+MSIX package")
    Check "start-at-login deferred to Windows" $true ($status -match "managed by Windows")

    Write-Host "== --autostart refuses, rather than writing a key nobody reads =="
    $out2 = Join-Path $env:RUNNER_TEMP "capslang-packaged-autostart.txt"
    if (Test-Path $out2) { Remove-Item $out2 -Force }
    Invoke-CommandInDesktopPackage -PackageFamilyName $pkg.PackageFamilyName `
        -AppId "CapsLang" `
        -Command "$env:ComSpec" `
        -Args "/c `"`"$exe`" --autostart on > `"$out2`" 2>&1`""
    for ($i = 0; $i -lt 20 -and -not (Test-Path $out2); $i++) { Start-Sleep -Milliseconds 500 }
    $auto = if (Test-Path $out2) { (Get-Content $out2 -Raw).Trim() } else { "(no output)" }
    Write-Host "  said: $auto"
    Check "declines to manage startup itself" $true ($auto -match "Windows manages this")

    Write-Host "== The unpackaged build is unaffected =="
    if (Test-Path (Join-Path $root $UnpackagedExe)) {
        $out3 = Join-Path $env:RUNNER_TEMP "capslang-standalone-status.txt"
        if (Test-Path $out3) { Remove-Item $out3 -Force }
        $p = Start-Process (Join-Path $root $UnpackagedExe) -ArgumentList "--status" `
            -RedirectStandardOutput $out3 -Wait -NoNewWindow -PassThru
        $plain = (Get-Content $out3 -Raw).Trim()
        Write-Host "  $($plain -split "`n" | Select-Object -First 1)"
        Check "reports standalone" $true ($plain -match "Installed as:\s+standalone")
        Check "exit code says not running" 1 $p.ExitCode
    } else {
        Write-Host "  skipped, no unpackaged build present"
    }
}
finally {
    Write-Host "== Clean up =="
    $pkg = Get-AppxPackage | Where-Object { $_.Name -like "*CapsLang*" } | Select-Object -First 1
    if ($pkg) {
        Remove-AppxPackage $pkg.PackageFullName -ErrorAction SilentlyContinue
        Write-Host "  removed"
    }
    Pop-Location
}

if ($failures.Count -gt 0) {
    Write-Host ""
    Write-Host "$($failures.Count) check(s) failed: $($failures -join ', ')" -ForegroundColor Red
    exit 1
}
Write-Host ""
Write-Host "All packaged-build checks passed." -ForegroundColor Green
